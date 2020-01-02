# rsp10 - (Rust Server Pages) - a Rust Web Framework

This is a work in progress rewrite of the web framework
the ideas of which I am using in multiple non-opensource
personal projects.

I started working on the ideas for this a couple of years ago,
when the choice of frameworks was pretty scarce, and also
the ones that were there, were requiring stable rust.

This project has a few general principles behind:

1. No unsafe {} blocks.
2. Only stable Rust.
3. Completely stateless on the server side.
4. Clear separation between Access, Logic and Presentation layers.
5. Allow to customize each layer, but give sane defaults.

DISCLAIMER: this is work in progress and interfaces are
subject to change.

# Quick example

Before running the examples, make sure your setup has the necessary prerequisites,
on Ubuntu 18.04 it is:

```
sudo apt-get install build-essential libsqlite3-dev libpq-dev
```

If you do not have these installed, the compilation will fail.
When you have them installed, running the example looks as follows:

```
ubuntu@host:~/rsp10$ cargo run --example simple
    Finished dev [unoptimized + debuginfo] target(s) in 0.07s
     Running `target/debug/examples/simple`
HTTP server for Simple Example starting on 127.0.0.1:4480
```

then connect the browser to http://127.0.0.1:4480/ - or, if
you are running the example on a different machine, bind
to all addresses:

```
ubuntu@host:~/rsp10$ BIND_IP=0.0.0.0 cargo run --example simple
    Finished dev [unoptimized + debuginfo] target(s) in 0.08s
     Running `target/debug/examples/simple`
HTTP server for Simple Example starting on 0.0.0.0:4480
```

You will be prompted to login (user "user" and "pass") and then
you will see the example 'interactive' page which has a few
input elements and allows to get the idea of what this is all about.

Did it work ? Interested to know how ? Here's some more to it....

# Foundational Ideas

The basic idea is that each web page can be represented by three
data elements:

1. *State Key*: This is a (maybe optional) set of arguments passed via the query string that
   define the initial state for the page when it is being loaded. A very simple key is an
   Option<i32> being the optional ID of an entity to edit.
2. *Initial State*: This is a record containing the state of the page, which is initially
   retrieved based on *State Key* before being sent to renderer.
3. *State*: This is the *current* state, which may or may not be different from
   the *Initial State*, the difference is expected to be due to the user changing it
   by typing in the text elements, selecting different dropdowns, etc.

However, what happens if someone modifies the data in question in the background ?
For this reason we need a fourth data field: *Current Initial State* - this is the "Initial State",
however, freshly recalculated before each pass of business logic.

In order to perform any business logic we also need a fifth component, and that is *Event*.

Kept together, these five elements allow to perform any business logic in a completely
stateless manner - which is a very useful property. It allows less logic for load balancing, as well as allows
to survive the service restarts and (potentially) upgrades.

# Life Cycle of a Page

1. The browser performs the GET request, optionally supplying parameters for the *State Key*.

2. The server converts the query string arguments into *State Key*. It uses two parts for it: the definition of the page key,
and optionally the method for converting the querystring arguments into the key. The state key for a page
can be defined as follows (in this example the key is a simple integer):

```rust
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct KeyI32 {
    id: Option<i32>,
}
```

Note, that with the above definition, passing the "id=XXX" (where XXX is something that parses as i32) will automatically populate the
state key, however it is possible to have the custom method as well, for example like this:

```rust
    fn get_key(
        auth: &MyPageAuth,
        args: &HashMap<String, Vec<String>>,
        maybe_state: &Option<PageState>,
    ) -> Option<KeyI32> {
        Some(KeyI32 {
            id: args.get("id").map_or(None, |x| x[0].parse::<i32>().ok()),
        })
    }
```

3. Initial request: The server uses the key to retrieve the *Initial State*, which again has two parts: a struct holding it and method
that populates it. Here is a sample struct:

```rust
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PageState {
    message: String,
    dd_testing: i32,
    txt_text_message: String,
    cbTestCheck: bool,
    ddMyDropdown: i32,
}
```

The method that returns the *Initial State*, based on the *State Key* must be idempotent:

```rust
    fn get_state(req: &mut Request, auth: &MyPageAuth, key: KeyI32) -> PageState {
        println!("default state for PageState with key: {:?}", &key);
        PageState {
            dd_testing: -1,
            txt_text_message: "test".to_string(),
            ddMyDropdown: key.id.unwrap_or(-1),
            cbTestCheck: true,
            ..Default::default()
        }
    }

```

If we are processing an update, then the form data will contain the *Initial State*, as well as *State*, which will be
filled in from the form data. The *Current Initial State* will be still freshly filled as above.

4. Handle the event. The event handler is called on each page update, including the initial load. Based on
the event, as well as the three state records, it may alter the *State* to its liking, as well as return *Action* to perform:
e.g. render the page, redirect to a different URL, etc.

```rust
    fn event_handler(ri: RspInfo<Self, KeyI32, MyPageAuth>) -> RspEventHandlerResult<Self, KeyI32> {
        let mut action = rsp10::RspAction::Render;
        let mut initial_state = ri.initial_state;
        let mut state = ri.state;

        if ri.event.event == "submit" {
            state.message = "".to_string();
            if !ri.state_none {
                let ev = ri.event;
                let tgt = &ev.target[..];
                match tgt {
                    "_eq" => {
                        state.txt_text_message =
                            format!("Pressed eq when state is {}", state.dd_testing);
                    }
                    "_lt" => {
                        state.dd_testing = state.dd_testing - 1;
                    }
                    "_gt" => {
                        if state.dd_testing == -1 {
                            state.message = format!("Select a value from the right dropdown first");
                        } else {
                          state.dd_testing = state.dd_testing + 1;
                        }
                    }
                    _ => {}
                }
            }
        }
        RspEventHandlerResult {
            initial_state,
            state,
            action,
        }
    }

```

5. Now the server can populate the data that will be used to render the template.
If the page does not contain any interactive elements, then it is not necessary
to define it, but since most of the pages actually do interact, you will define it,
something along these lines:

```rust
    fn fill_data(ri: RspInfo<Self, KeyI32, MyPageAuth>) -> RspFillDataResult<Self> {
        let mut modified = false;
        let mut ri = ri;
        let mut gd = RspDataBuilder::new();
        let real_key = ri.key.id.unwrap_or(-1);
        println!("{:?}", &ri.state);

        rsp10_button!(btnTest, "Test button" => gd);
        rsp10_select!(dd_testing, dbh_get_dropdown(ri.state.dd_testing), ri => gd, modified);
        rsp10_select!(ddMyDropdown, dbh_get_dropdown(real_key), ri => gd, modified);
        rsp10_text!(txt_text_message, ri => gd, modified);
        rsp10_check!(gd, cbTestCheck, ri, modified);
        rsp10_data!(modified => gd);

        Self::fill_data_result(ri, gd)
    }
```

This method effectively translates the higher-level abstractions of the state into
more visual data for the template rendering - dropdowns, checkboxes, text elements,
and just simple data. 

You will notice most of the operations are hidden behind macros - this is to minimize
the clutter, because behind the scenes the "state.SomeElement" value, which may be
an i32, for example, is rendered into a "SomeElement" Rc<RefCell<HtmlElement>>, which 
can be modified within the *fill_data()* code, if the complex UI element interactions
require it.

After finishing the preparation the RspDataBuilder object
is passed to the *Self::fill_data_result()* along with the "RspInfo" structure
(which contains a lot of interesting data about the request), which compiles
the Mustache data builder and returns the RspFillDataResult, which is used to render the templates.

6. Compile the Mustache template file. The file name is normally derived automatically,
but you can override it on a per-page basis. Also - for simplicity of debugging the compile
currently happens on each page load, but it is trivial to compile the templates once upon the start.
The option to do so will may be some time in the future.

The typical template file will contain HTML forms, with the template looking as follows:

```
<form method="post">
{{#btnTest}} {{> html/submit}} {{/btnTest}}
{{#ddMyDropdown}} {{>html/select}} {{/ddMyDropdown}}
{{#dd_testing}} {{>html/select}} {{/dd_testing}}
{{#cbTestCheck}} {{> html/checkbox }} {{/cbTestCheck}}
{{#txt_text_message}} {{> html/text }} {{/txt_text_message}}
<input type="hidden" name="initial_state_json" value="{{initial_state_json}}">
<input type="hidden" name="state_json" value="{{state_json}}">
<input type="submit" name="submit_lt" value="<">
<input type="submit" name="submit_eq" value="=">
<input type="submit" name="submit_gt" value=">">
</form>
```

Notice the '{{#foo}} ... {{/foo}}' pairs, with {{> html/something}} inside.
While it looks like some cool markup - this is simply Mustache syntax to "dive in"
one level into the element. It allows to have a very uniform yet easily customizable
look and feel for the elements. As you see, you can also code regular HTML with no template
data whatsoever.

However, note the two fields "*initial_state_json*" and "*state_json*" - they are
essential for the correct functioning, and carry the state information about the page.

7. The rendered page is sent to the user.

8. User performs some manipulations, the client side code potentially does something as well,
and eventualy a changed data is being submitted. At this point the cycle repeats from the beginning.


You will notice that current implementation is completely Javascript-free: this is obviously
not the final state of affairs, but one of the goals of this framework was graceful fallback,
and javascript-free operation with the server side completely controlling the data flow.

In the future more client-side functionality will be added.

# Authentication

I have completely omitted discussing the question of access control, however a curious reader might
have noticed the "MyPageAuth" type.

Authentication is implemented via a trait, which returns Result<AuthObject, String> - with the successful
result being the auth object, and the error containing a string with the URL to redirect to. The simplest
authentication is no authentication:

```rust
pub struct NoPageAuth {}
impl rsp10::RspUserAuth for NoPageAuth {
    fn from_request(_req: &mut iron::Request) -> Result<NoPageAuth, String> {
        Ok(NoPageAuth {})
    }
}
```

In case the authentication layer returns the error, the processing of the request stops and
a redirect to the supplied login URL is issued. This way, once you specify the auth type in
the resource, you do not have to worry about it - you simply get the auth object that you can
query.





