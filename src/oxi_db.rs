use oxigraph::sparql::{QueryResultsFormat, QueryResults};
use oxigraph::model::*;
use oxigraph::io::GraphFormat;
use std::error::Error;
use std::io::{BufReader, BufWriter};
use std::str;
use oxigraph::store::{StorageError, Store};

// IMPORTANT! FOR Server version use in CL:
// oxigraph_server --location ./oxi_server_data serve

const SELECT_EVENTS: &str = "PREFIX time: <http://www.w3.org/2006/time#>
PREFIX baseUrl: <http://example.data/event/>
SELECT ?e1 ?st1 ?end1 WHERE {  ?e1 _START_DATE_PLACEHOLDER_ ?st1 ; _END_DATE_PLACEHOLDER_ ?end1 .}";

const CONSTRUCT_ALL_FOR_GIVEN_EVENT1: &str = "PREFIX xsd: <http://www.w3.org/2001/XMLSchema#>
PREFIX time: <http://www.w3.org/2006/time#>
PREFIX baseUrl: <http://example.data/event/>
CONSTRUCT {?e1 ?timeInterval ?e2.}
WHERE {
    ?e2 _START_DATE_PLACEHOLDER_ ?st2 ;
        _END_DATE_PLACEHOLDER_ ?end2 . 
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
    ?e2 _START_DATE_PLACEHOLDER_ ?st2 ;
        _END_DATE_PLACEHOLDER_ ?end2 . 
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
    SELECT ?e1 ?e2 ?timeInterval WHERE {?e2 _START_DATE_PLACEHOLDER_ ?st2 ; _END_DATE_PLACEHOLDER_ ?end2 . BIND(_E1_PLACEHOLDER_  AS ?e1).  FILTER(?e1 != ?e2). FILTER(_ST1_PLACEHOLDER_ < ?st2 && _END1_PLACEHOLDER_ > ?st2 && _END1_PLACEHOLDER_ < ?end2)  . BIND(time:intervalOverlaps AS ?timeInterval).  
    }}
    UNION
    {SELECT ?e1 ?e2  ?timeInterval WHERE {?e2 _START_DATE_PLACEHOLDER_ ?st2 ; _END_DATE_PLACEHOLDER_ ?end2 . BIND(_E1_PLACEHOLDER_  AS ?e1). FILTER(?e1 != ?e2). FILTER(_END1_PLACEHOLDER_ = ?st2) .BIND(time:intervalMeets AS ?timeInterval). 
    }}
    UNION
    {SELECT ?e1 ?e2 ?timeInterval WHERE {?e2 _START_DATE_PLACEHOLDER_ ?st2 ; _END_DATE_PLACEHOLDER_ ?end2 . BIND(_E1_PLACEHOLDER_  AS ?e1). FILTER(?e1 != ?e2). FILTER(_ST1_PLACEHOLDER_ < ?st2 && _END1_PLACEHOLDER_ > ?end2)  BIND(time:intervalContains AS ?timeInterval). 
    }}
    UNION
    {SELECT ?e1 ?e2  ?timeInterval WHERE {?e2 _START_DATE_PLACEHOLDER_ ?st2 ; _END_DATE_PLACEHOLDER_ ?end2 . BIND(_E1_PLACEHOLDER_  AS ?e1).  FILTER(?e1 != ?e2) .
    FILTER(_ST1_PLACEHOLDER_ >= ?st2 && _END1_PLACEHOLDER_ <= ?end2 && !(_ST1_PLACEHOLDER_ = ?st2 && _END1_PLACEHOLDER_ = ?end2)).
    BIND(time:intervalIn AS ?timeInterval)
    }}
  }
} LIMIT 100000";

pub fn process_data(data: &[u8], start_date_prop_name: &str, end_date_prop_name: &str) -> Result<String,Box<dyn Error>>
{
    if let "" = start_date_prop_name {Err("start_date_prop_name must not be empty.")?}
    if let "" = end_date_prop_name {Err("end_date_prop_name must not be empty.")?}

    let reader = BufReader::new(data);
    let store = Store::new()?;
    store.load_graph(BufReader::new(reader), GraphFormat::Turtle, GraphNameRef::DefaultGraph, None)?;
    let mut res = "".to_owned();
    if let QueryResults::Solutions(sorted_events) = store.query(&replace_placeholders(SELECT_EVENTS, start_date_prop_name, end_date_prop_name))? {//TODO: this whole thing should be a separate fn - if any error a proper cleanup (store.clear()) should be called!
        for solution in sorted_events {
            let event = solution?;
            let ev_res = check_event(&store,
                match event.get("e1").ok_or("No ?e1 in SPARQL. Check the SPARQL query!")? {Term::NamedNode(ev1) => Ok(ev1), _ => Err("NamedNode expected.")}?,
                match event.get("st1").ok_or("No ?st1 in SPARQL. Check the SPARQL query!")? {Term::Literal(start_time) => Ok(start_time), _ => Err("Literal expected")}?,
                match event.get("end1").ok_or("No ?end1 in SPARQL. Check the SPARQL query!")? {Term::Literal(end_time) => Ok(end_time), _ => Err("Literal expected")}?, 
                start_date_prop_name, end_date_prop_name);
            res.push_str(&ev_res?);
        }
    }

    store.clear()?;
    return Ok(res);
}

fn replace_placeholders(query: &str, start_date_prop_name: &str, end_date_prop_name: &str) -> String
{
    return query.replace("_START_DATE_PLACEHOLDER_", start_date_prop_name)
                        .replace("_END_DATE_PLACEHOLDER_", end_date_prop_name);
}

pub fn check_event(store: &Store, event: &NamedNode, start_time: &Literal, end_time:&Literal, start_date_prop_name: &str, end_date_prop_name: &str) -> Result<String, Box<dyn Error>> {
    let start_time_st = format!("\"{}\"^^<{}>", start_time.value(), start_time.datatype().as_str());
    let end_time_st = format!("\"{}\"^^<{}>", end_time.value(), end_time.datatype().as_str());
    let event_url = format!("<{}>", event.as_str());
    let construct_query = replace_placeholders(CONSTRUCT_ALL_FOR_GIVEN_EVENT, start_date_prop_name, end_date_prop_name)
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
    use std::fs;

    // Note this useful idiom: importing names from outer (for mod tests) scope.
    use super::*;

    const DATA:&str = "@prefix xsd: <http://www.w3.org/2001/XMLSchema#> .
    @prefix baseUrl: <http://example.data/event/> .
        baseUrl:de105920-461f-495c-bf60-293fc2a45d81 baseUrl:id \"de105920-461f-495c-bf60-293fc2a45d80\"; baseUrl:startDate \"2001-12-16T02:50:21Z\"^^xsd:dateTime; baseUrl:endDate \"2001-12-17T21:41:58Z\"^^xsd:dateTime.
        baseUrl:de105920-461f-495c-bf60-293fc2a45d82 baseUrl:id \"de105920-461f-495c-bf60-293fc2a45d80\"; baseUrl:startDate \"2001-12-15T02:50:20Z\"^^xsd:dateTimeStamp; baseUrl:endDate \"2001-12-17T21:45:58Z\"^^xsd:dateTimeStamp.";

    #[test]
    fn test_process_data_multi_string() {
        let r = process_data(DATA.as_bytes(), "baseUrl:startDate", "baseUrl:endDate");
        // println!("RES:{}", r.unwrap_err());
        let r = r.unwrap();
        assert_ne!(r, "");  
        println!("RES:{}", r);
    }

    #[test]
    fn test_process_data_multi_file() {
        let contents = fs::read_to_string("/home/nowar/my_projects/wasm-oxi-time/src/event.ttl").expect("Should have been able to read the file");
        let r = process_data(contents.as_bytes(), "baseUrl:startDate", "baseUrl:endDate");
        // println!("RES:{}", r.unwrap_err());
        assert_eq!(r.is_ok(), true);  
        let r = r.unwrap();
        assert_ne!(r, "");  
        println!("RES: {}", r);
    }

}