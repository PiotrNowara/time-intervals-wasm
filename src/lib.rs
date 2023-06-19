use wasm_bindgen::prelude::*;
mod oxi_db;
mod csv_to_rdf;

// How to build?
//  cargo build --target wasm32-unknown-unknown
//  wasm-bindgen target/wasm32-unknown-unknown/debug/wasm_oxi_time.wasm --out-dir ./web/ --target web
//  wasm-bindgen target/wasm32-unknown-unknown/release/wasm_oxi_time.wasm --out-dir ./web/ --target web
//  wasm-bindgen target/wasm32-unknown-unknown/release/wasm_oxi_time.wasm --out-dir ./docs/ --target web

// HOW to run? Go to ./web and run Python server: python3 -m http.server  

// Assumptions for CSV transformations:
// id column has to be present - it will be used to construct event URL: <baseUrl/uuid/id>. It will not be present in the transformed RDF data.
// start_time,end_time prefixed columns have to be present (can be named just: start_time,end_time)
// SPARQL is type sensitive so using "2022-08-10" as a xsd:dateTime will not work (pattern will not be matched) - exact timestamps or dates have to be provided. Data will be transformed according to provided values.

#[wasm_bindgen]
pub fn analyze_file(file_input: web_sys::HtmlInputElement) -> Result<(), JsError> {

    log("Starting processing... v.0.1.3");
    let filelist = file_input.files().expect_throw("No file given.");
    filelist.get(0).expect_throw("Please select a valid file");
    
    let file = filelist.get(0).ok_or(JsError::new("Unable to retrieve a file from HTML component."))?;

    let file_reader : web_sys::FileReader = match web_sys::FileReader::new() {
        Ok(f) => f,
        Err(e) => {
            alert("There was an error creating a file reader");
            return Err(JsError::new(&format!("Error creating a file reader: {:?}", e)));
        }
    };

    let fr_c = file_reader.clone(); 
    // create onLoadEnd callback
    let filename = file.name();
    log(&format!("Submitted file: {}", filename));
    let onloadend_cb = Closure::wrap(Box::new(move |_e: web_sys::ProgressEvent| {
        let array = js_sys::Uint8Array::new(&fr_c.result().expect_throw("Error when processing CSV data input string: {}"));
        let mut slice = vec![0; array.length() as usize];
        array.copy_to(&mut slice[..]); //TODO: if this does not work then use array.to_vec()

        // handler code
        log(&format!("In process... file: {}", filename));
        if filename.ends_with(".csv")
        {
            log("Parsing CSV file.");
            slice = csv_to_rdf::csv_to_rdf(&slice)
                .map_err(|e| JsError::new(&format!("Error when processing CSV data input string: {}", e.to_string())))?;
        }
        // log(&format!("RDF content: {:?}", std::str::from_utf8(&slice).unwrap()));
        let res_str = oxi_db::process_data(&slice)
            .map_err(|e| JsError::new(&format!("Error when processing RDF data input string: {}", e.to_string())))?;
        handleResult(&res_str);
        Ok(())
    }) as Box<dyn Fn(web_sys::ProgressEvent) -> Result<(), JsError>>);

    file_reader.set_onloadend(Some(onloadend_cb.as_ref().unchecked_ref()));
    file_reader.read_as_array_buffer(&file).expect_throw("Unable to read the submitted file.");
    onloadend_cb.forget();
    Ok(())
}

#[wasm_bindgen]
pub fn analyze_string(input_string: String) -> Result<(), JsError> {
    log("Starting processing... v.0.1.3");
    if input_string.is_empty() {
        alert("Please enter a non-empty RDF data string.");
        JsError::new("Please enter a non-empty RDF data string.");
    }
    log("In process...");
    let res_str = oxi_db::process_data(input_string.as_bytes())
        .map_err(|e| JsError::new(&format!("Error when processing RDF data input string: {}", e.to_string())))?;
    handleResult(&res_str);
    Ok(())
}

#[wasm_bindgen]
pub fn analyze_csv_string(input_string: String) -> Result<(), JsError> {
    log("Starting processing... v.0.1.3");
    if input_string.is_empty() {
        alert("Please enter a non-empty CSV data string.");
        JsError::new("Please enter a non-empty CSV data string.");
    }
    // handler code
    log("In process...");
    let st = csv_to_rdf::csv_to_rdf(input_string.as_bytes())
        .map_err(|e| JsError::new(&format!("Error when processing CSV data input string: {}", e.to_string())))?;
    // log(&format!("RDF content: {:?}", std::str::from_utf8(&st).unwrap()));//TODO: delete after testing
    let res_str = oxi_db::process_data(&st)
        .map_err(|e| JsError::new(&format!("Error when processing RDF data input string: {}", e.to_string())))?;
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
