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


// const CONSTRUCT_OVERLAPS:&str = "PREFIX time: <http://www.w3.org/2006/time#>
// PREFIX baseUrl: <http://example.data/event/>
// Construct {?e1 time:intervalOverlaps ?e2}
// WHERE {
// #GRAPH <http://time.example/data> 
//     ?e1 baseUrl:startDate ?st1 ;
//         baseUrl:endDate ?end1 .
//     ?e2 baseUrl:startDate ?st2 ;
//         baseUrl:endDate ?end2 . 
//     FILTER (?st1 < ?st2 && ?end1 > ?st2 && ?end1 < ?end2)

// } LIMIT 50000
// ";

const CONSTRUCT_ALL:&str = "PREFIX time: <http://www.w3.org/2006/time#>
PREFIX baseUrl: <http://example.data/event/>
CONSTRUCT {?e1 ?timeInterval ?e2.}
WHERE {
    ?e1 baseUrl:startDate ?st1 ;
        baseUrl:endDate ?end1 .
    ?e2 baseUrl:startDate ?st2 ;
        baseUrl:endDate ?end2 . 
  BIND(?st1 < ?st2 && ?end1 > ?st2 && ?end1 < ?end2 AS ?intervalOverlaps)
  BIND(?e1 != ?e2 && ?end1 = ?st2 AS ?intervalMeets)
  BIND(?st1 < ?st2 && ?end1 > ?end2 AS ?intervalContains)
  BIND(?st1 >= ?st2 && ?end1 <= ?end2 && !(?st1 = ?st2 && ?end1 = ?end2) AS ?intervalIn)
  FILTER (?intervalOverlaps || ?intervalMeets || ?intervalContains || ?intervalIn)
  BIND(IF(?intervalOverlaps, time:intervalOverlaps, 
      	IF(?intervalMeets, time:intervalMeets, 
        IF(?intervalContains, time:intervalContains, time:intervalIn))) AS ?timeInterval)
} LIMIT 50000
";

const SELECT_EVENTS: &str = "PREFIX time: <http://www.w3.org/2006/time#>
PREFIX baseUrl: <http://example.data/event/>
SELECT ?e1 ?st1 ?end1 WHERE {  ?e1 baseUrl:startDate ?st1 ;  baseUrl:endDate ?end1 .}";

const CONSTRUCT_ALL_FOR_GIVEN_EVENT1: &str = "PREFIX xsd: <http://www.w3.org/2001/XMLSchema#>
PREFIX time: <http://www.w3.org/2006/time#>
PREFIX baseUrl: <http://example.data/event/>
CONSTRUCT {?e1 ?timeInterval ?e2.}
WHERE {
    ?e2 baseUrl:startDate ?st2 ;
        baseUrl:endDate ?end2 . 
  BIND(_E1_PLACEHOLDER_  AS ?e1)
  FILTER(?e1 != ?e2)
  BIND(_ST1_PLACEHOLDER_ < ?st2 && _END1_PLACEHOLDER_ > ?st2 && _END1_PLACEHOLDER_ < ?end2 AS ?intervalOverlaps)
  BIND(_END1_PLACEHOLDER_ = ?st2 AS ?intervalMeets)
  BIND(_ST1_PLACEHOLDER_ < ?st2 && _END1_PLACEHOLDER_ > ?end2 AS ?intervalContains)
  BIND(_ST1_PLACEHOLDER_ >= ?st2 && _END1_PLACEHOLDER_ <= ?end2 && !(_ST1_PLACEHOLDER_ = ?st2 && _END1_PLACEHOLDER_ = ?end2) AS ?intervalIn)
  FILTER (?intervalOverlaps || ?intervalMeets || ?intervalContains || ?intervalIn)
  BIND(IF(?intervalOverlaps, time:intervalOverlaps, 
      	IF(?intervalMeets, time:intervalMeets, 
        IF(?intervalContains, time:intervalContains, time:intervalIn))) AS ?timeInterval)
} LIMIT 100000";

const CONSTRUCT_ALL_FOR_GIVEN_EVENT: &str = "PREFIX xsd: <http://www.w3.org/2001/XMLSchema#>
PREFIX time: <http://www.w3.org/2006/time#>
PREFIX baseUrl: <http://example.data/event/>
CONSTRUCT {?e1 ?timeInterval ?e2.}
WHERE {
    ?e2 baseUrl:startDate ?st2 ;
        baseUrl:endDate ?end2 . 
  BIND(_E1_PLACEHOLDER_  AS ?e1)
  FILTER(?e1 != ?e2)
  BIND(IF(_ST1_PLACEHOLDER_ < ?st2 && _END1_PLACEHOLDER_ > ?st2 && _END1_PLACEHOLDER_ < ?end2, time:intervalOverlaps, 
      	IF(_ST1_PLACEHOLDER_ < ?st2 && _END1_PLACEHOLDER_ > ?end2, time:intervalContains, 
        IF(_END1_PLACEHOLDER_ = ?st2, time:intervalMeets, 
        IF(_ST1_PLACEHOLDER_ >= ?st2 && _END1_PLACEHOLDER_ <= ?end2 && !(_ST1_PLACEHOLDER_ = ?st2 && _END1_PLACEHOLDER_ = ?end2), time:intervalIn, -1 )))) AS ?timeInterval)
  FILTER(?timeInterval != -1)
} LIMIT 100000";

// slightly slower
const C2:&str = "
PREFIX xsd: <http://www.w3.org/2001/XMLSchema#>
PREFIX time: <http://www.w3.org/2006/time#>
PREFIX baseUrl: <http://example.data/event/>
CONSTRUCT {?e1 ?timeInterval ?e2.}
WHERE {
  {
    {
    SELECT ?e1 ?e2 ?timeInterval WHERE {?e2 baseUrl:startDate ?st2 ; baseUrl:endDate ?end2 . BIND(_E1_PLACEHOLDER_  AS ?e1).  FILTER(?e1 != ?e2). FILTER(_ST1_PLACEHOLDER_ < ?st2 && _END1_PLACEHOLDER_ > ?st2 && _END1_PLACEHOLDER_ < ?end2)  . BIND(time:intervalOverlaps AS ?timeInterval).  
    }}
    UNION
    {SELECT ?e1 ?e2  ?timeInterval WHERE {?e2 baseUrl:startDate ?st2 ; baseUrl:endDate ?end2 . BIND(_E1_PLACEHOLDER_  AS ?e1). FILTER(?e1 != ?e2). FILTER(_END1_PLACEHOLDER_ = ?st2) .BIND(time:intervalMeets AS ?timeInterval). 
    }}
    UNION
    {SELECT ?e1 ?e2 ?timeInterval WHERE {?e2 baseUrl:startDate ?st2 ; baseUrl:endDate ?end2 . BIND(_E1_PLACEHOLDER_  AS ?e1). FILTER(?e1 != ?e2). FILTER(_ST1_PLACEHOLDER_ < ?st2 && _END1_PLACEHOLDER_ > ?end2)  BIND(time:intervalContains AS ?timeInterval). 
    }}
    UNION
    {SELECT ?e1 ?e2  ?timeInterval WHERE {?e2 baseUrl:startDate ?st2 ; baseUrl:endDate ?end2 . BIND(_E1_PLACEHOLDER_  AS ?e1).  FILTER(?e1 != ?e2) .
    FILTER(_ST1_PLACEHOLDER_ >= ?st2 && _END1_PLACEHOLDER_ <= ?end2 && !(_ST1_PLACEHOLDER_ = ?st2 && _END1_PLACEHOLDER_ = ?end2)).
    BIND(time:intervalIn AS ?timeInterval)
    }}
  }
} LIMIT 100000";

// original way - quite inefficient
pub fn process_data_with_single_query(data: &[u8]) -> Result<String,Box<dyn Error>>
{
    let reader = BufReader::new(data);
    let store = Store::new()?;
    store.load_graph(BufReader::new(reader), GraphFormat::Turtle, GraphNameRef::DefaultGraph, None)?;

    let res_graph = store.query(CONSTRUCT_ALL)?;
    let mut buf = Vec::new();
    res_graph.write_graph(&mut buf, GraphFormat::Turtle)?;
    store.clear()?;
    return Ok(String::from_utf8(buf)?);
}

pub fn process_data(data: &[u8]) -> Result<String,Box<dyn Error>>
{
    let reader = BufReader::new(data);
    let store = Store::new()?;
    store.load_graph(BufReader::new(reader), GraphFormat::Turtle, GraphNameRef::DefaultGraph, None)?;
    println!("Events retrieved");
    let mut res = "".to_owned();
    if let QueryResults::Solutions(sorted_events) = store.query(SELECT_EVENTS)? {//TODO: this whole thing should be a separate fn - if any error a proper cleanup (store.clear()) should be called!
     for solution in sorted_events {
        let event = solution?;
        let ev_res = check_event(&store,
            match event.get("e1").ok_or("No ?e1 in SPARQL. Check the SPARQL query!")? {Term::NamedNode(ev1) => Ok(ev1), _ => Err("NamedNode expected.")}?,
            match event.get("st1").ok_or("No ?st1 in SPARQL. Check the SPARQL query!")? {Term::Literal(start_time) => Ok(start_time), _ => Err("Literal expected")}?,
            match event.get("end1").ok_or("No ?end1 in SPARQL. Check the SPARQL query!")? {Term::Literal(end_time) => Ok(end_time), _ => Err("Literal expected")}?);
        res.push_str(&ev_res?);
        //  println!("{:?}", solution?.get("s"));
     }
   }

    store.clear()?;
    return Ok(res);
}

pub fn check_event(store: &Store, event: &NamedNode, start_time: &Literal, end_time:&Literal) -> Result<String, Box<dyn Error>> {
    let start_time_st = format!("\"{}\"^^<{}>", start_time.value(), start_time.datatype().as_str());
    let end_time_st = format!("\"{}\"^^<{}>", end_time.value(), end_time.datatype().as_str());
    let event_url = format!("<{}>", event.as_str());
    let construct_query = CONSTRUCT_ALL_FOR_GIVEN_EVENT
        .replace("_ST1_PLACEHOLDER_", start_time_st.as_str())
        .replace("_END1_PLACEHOLDER_", end_time_st.as_str())
        .replace("_E1_PLACEHOLDER_", event_url.as_str());
    // println!("Query: {}", construct_query);
    let res_graph = store.query(&construct_query)?;
    let mut buf = Vec::new();
    res_graph.write_graph(&mut buf, GraphFormat::Turtle)?;
    return Ok(String::from_utf8(buf)?);
}

#[cfg(test)]
mod tests {
    // Note this useful idiom: importing names from outer (for mod tests) scope.
    use super::*;

    const DATA:&str = "@prefix xsd: <http://www.w3.org/2001/XMLSchema#> .
    @prefix baseUrl: <http://example.data/event/> .
        baseUrl:de105920-461f-495c-bf60-293fc2a45d80 baseUrl:id \"de105920-461f-495c-bf60-293fc2a45d80\"; baseUrl:startDate \"2001-12-16T02:50:21Z\"^^xsd:dateTime; baseUrl:endDate \"2001-12-17T21:41:58Z\"^^xsd:dateTime.
        baseUrl:de105920-461f-495c-bf60-293fc2a45d80 baseUrl:id \"de105920-461f-495c-bf60-293fc2a45d80\"; baseUrl:startDate \"2001-12-15T02:50:21Z\"^^xsd:dateTime; baseUrl:endDate \"2001-12-17T21:45:58Z\"^^xsd:dateTime.";

    #[test]
    fn test_process_data_multi_string() {
        let r = process_data(DATA.as_bytes());
        // println!("RES:{}", r.unwrap_err());
        assert_eq!(r.is_ok(), true);
    }

    // #[test]
    // fn test_process_data_multi_file() {
    //     let contents = fs::read_to_string("/home/nowar/my_projects/wasm-oxi-time/src/event3.ttl").expect("Should have been able to read the file");
    //     let r = process_data_multi(contents.as_bytes());
    //     // println!("RES:{}", r.unwrap_err());
    //     assert_eq!(r.is_ok(), true);  
    //     assert_ne!(r.unwrap(), "");  
    //     // println!("RES: {}", r.unwrap());
    // }

}