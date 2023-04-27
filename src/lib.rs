use wasm_bindgen::prelude::*;
mod oxi_db;

// How to build?
//  cargo build --target wasm32-unknown-unknown
//  wasm-bindgen target/wasm32-unknown-unknown/debug/wasm_oxi_time.wasm --out-dir ./web/ --target web
//  wasm-bindgen target/wasm32-unknown-unknown/release/wasm_oxi_time.wasm --out-dir ./web/ --target web

//HOW to run? Go to ./web and run Python server: python3 -m http.server  

#[wasm_bindgen]
pub fn analyze_file(file_input: web_sys::HtmlInputElement) -> Result<(), JsError> {
    //Check the file list from the input
    // file_input.fi
    let filelist = file_input.files().expect_throw("No file given.");
    //Do not allow blank inputs
    if filelist.length() < 1 {
        alert("Please select at least one file.");
        JsError::new("Please select at least one file.");
    }
    if filelist.get(0) == None {
        alert("Please select a valid file");
        JsError::new("Please select a valid file");
    }
    
    let file = filelist.get(0).unwrap();

    let file_reader : web_sys::FileReader = match web_sys::FileReader::new() {
        Ok(f) => f,
        Err(e) => {
            alert("There was an error creating a file reader");
            // println!(&JsValue::as_string(&e).expect("error converting jsvalue to string."));
            web_sys::FileReader::new().expect_throw("There was an error creating a file reader")
        }
    };

    let fr_c = file_reader.clone();
    // create onLoadEnd callback
    let onloadend_cb = Closure::wrap(Box::new(move |_e: web_sys::ProgressEvent| {
        let array = js_sys::Uint8Array::new(&fr_c.result().unwrap());
        let mut slice = vec![0; array.length() as usize];
        array.copy_to(&mut slice[..]); //TODO: if this does not work then use array.to_vec()

        // handler code
        log("In process...");
        let res_str = oxi_db::process_data(&slice).expect_throw("Error when analyzing data.");
        // alert(&res_str);
        handleResult(&res_str);
    }) as Box<dyn Fn(web_sys::ProgressEvent)>);

    file_reader.set_onloadend(Some(onloadend_cb.as_ref().unchecked_ref()));
    file_reader.read_as_array_buffer(&file).expect("blob not readable");
    onloadend_cb.forget();
    Ok(())
}

#[wasm_bindgen]
pub fn analyze_string(input_string: String) -> Result<(), JsError> {
    if input_string.is_empty() {
        alert("Please enter non-empty data string.");
        JsError::new("Please enter non-empty data string.");
    }
    // handler code
    log("In process...");
    let res_str = oxi_db::process_data(input_string.as_bytes()).expect_throw("Error when analyzing data.");
    // alert(&res_str);
    handleResult(&res_str);
    Ok(())
}


// Import 'window.alert' - external JS function can be accessed that way...
#[wasm_bindgen]
extern "C" {
    fn alert(s: &str);
}

// my custome function implemented on the JS side
#[wasm_bindgen]
extern "C" {
    fn handleResult(s: &str);
}

// log handling taken from: https://rustwasm.github.io/wasm-bindgen/examples/console-log.html
// TODO: add a Rust fn that would add a timestamp
#[wasm_bindgen]
extern "C" {
    // Use `js_namespace` here to bind `console.log(..)` instead of just
    // `log(..)`
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);

    // The `console.log` is quite polymorphic, so we can bind it with multiple
    // signatures. Note that we need to use `js_name` to ensure we always call
    // `log` in JS.
    #[wasm_bindgen(js_namespace = console, js_name = log)]
    fn log_u32(a: u32);

    // Multiple arguments too!
    #[wasm_bindgen(js_namespace = console, js_name = log)]
    fn log_many(a: &str, b: &str);
}
