use oxigraph::sparql::{QueryResultsFormat, QueryResults};
use oxigraph::model::*;
use oxigraph::io::GraphFormat;
use std::error::Error;
use std::fs::{File, self};
use std::io::{BufReader, BufWriter};
use std::time::SystemTime;
use std::str;
use oxigraph::store::{StorageError, Store};

use tempfile::tempfile;
use std::io::{self, Write};


// IMPORTANT! FOR Server version use in CL:
// oxigraph_server --location ./oxi_server_data serve

const SPARQL_OVERLAPS: &str = "PREFIX time: <http://www.w3.org/2006/time#>
PREFIX baseUrl: <http://example.data/event/>

INSERT
{
GRAPH <http://time.example/relations> {
    ?e1 time:intervalOverlaps ?e2 .
}
}
WHERE {
GRAPH <http://time.example/data> {
    ?e1 baseUrl:startDate ?st1 ;
        baseUrl:endDate ?end1 .
    ?e2 baseUrl:startDate ?st2 ;
        baseUrl:endDate ?end2 . 
    FILTER (?st1 < ?st2 && ?end1 > ?st2 && ?end1 < ?end2)
}
}
";

const CONSTRUCT_OVERLAPS:&str = "PREFIX time: <http://www.w3.org/2006/time#>
PREFIX baseUrl: <http://example.data/event/>

Construct {?e1 time:intervalOverlaps ?e2}
WHERE {
#GRAPH <http://time.example/data> 

    ?e1 baseUrl:startDate ?st1 ;
        baseUrl:endDate ?end1 .
    ?e2 baseUrl:startDate ?st2 ;
        baseUrl:endDate ?end2 . 
    FILTER (?st1 < ?st2 && ?end1 > ?st2 && ?end1 < ?end2)

} LIMIT 50000
";

pub fn process_data(data: &[u8]) -> Result<String,Box<dyn Error>>
{
    let reader = BufReader::new(data);
    let store = Store::new()?;
    store.load_graph(BufReader::new(reader), GraphFormat::Turtle, GraphNameRef::DefaultGraph, None);

    // let now = SystemTime::now(); //TODO: check why it panicks!
    let res_graph = store.query(CONSTRUCT_OVERLAPS)?;
    // println!("CONSTRUCT query completed in: {}", now.elapsed()?.as_millis());

    let mut buf = Vec::new();
    res_graph.write_graph(&mut buf, GraphFormat::Turtle)?;
    return Ok(String::from_utf8(buf)?);
}



