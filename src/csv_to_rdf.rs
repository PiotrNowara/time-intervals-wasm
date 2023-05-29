use std::{error::Error};
use uuid::Uuid;

const BASE_URL:&str = "http://example.data/event/";

pub fn csv_to_rdf(data: &[u8]) -> Result<Vec<u8>, Box<dyn Error>>
{
    let mut result: Vec<u8> = Vec::new();
    result.extend_from_slice(format!("@prefix xsd: <http://www.w3.org/2001/XMLSchema#> . @prefix baseUrl: <{}> . ", BASE_URL).as_bytes());
    let mut rdr = csv::Reader::from_reader(data);
    let headers = rdr.headers()?.clone();
    let mut i = 0;
    let mut start_time_attribute_index_extractor:Option<usize> = None;
    let mut end_time_attribute_index_extractor:Option<usize> = None;
    let mut id_attribute_index_extractor:Option<usize> = None;
    
    for col in &headers {
        if col.starts_with("start_time") {
            start_time_attribute_index_extractor = Some(i);
        }
        else if col.starts_with("end_time") {
            end_time_attribute_index_extractor = Some(i); 
        }
        else if col.eq("id") {
            id_attribute_index_extractor = Some(i); 
        }
        i += 1; 
    }

    let start_time_attribute_index = start_time_attribute_index_extractor.ok_or("No column starting with 'start_time' prefix found.")?;
    let end_time_attribute_index = end_time_attribute_index_extractor.ok_or("No column starting with 'end_time' prefix found.")?;
    let id_attribute_index = id_attribute_index_extractor.ok_or("No 'id' column found.")?;
    for row in rdr.records() {
        let raw_record = row?;
        let uuid = Uuid::new_v4().to_string();
        let mut s_row = format!("<{}{}/{}> ", BASE_URL, uuid, &raw_record[id_attribute_index]);
        for i in 0..headers.len() {
            if i == id_attribute_index { continue }

            let data_str = if i == start_time_attribute_index {
                format!("baseUrl:startDate \"{}\"^^{}", &raw_record[i], parse_time_expr(&raw_record[i]))
            }
            else if i ==  end_time_attribute_index {
                format!("baseUrl:endDate \"{}\"^^{}", &raw_record[i], parse_time_expr(&raw_record[i]))
            }
            else {
                format!("baseUrl:{} \"{}\"", &headers[i], &raw_record[i])
            };
            let ending = if i == headers.len()-1 {". "} else {"; "};
            s_row.push_str(&(data_str + ending));
        }
        result.extend_from_slice(s_row.as_bytes())
    }
    return Ok(result);
}

fn parse_time_expr(expr: &str) -> &'static str
{
    if expr.len() == 10 {
        return "xsd:date";
    }
    return "xsd:dateTime"
}