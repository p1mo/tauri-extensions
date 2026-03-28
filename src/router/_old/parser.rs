use std::str::FromStr;
use std::collections::HashMap;



pub type URLParams = HashMap<String, String>;
pub type URLQuerys = Option<HashMap<String, String>>;



#[derive(Debug)]
pub struct URLInfo {
    params: URLParams,
    querys: URLQuerys
}

impl URLInfo {
    
    /// ### Get query with key
    /// 
    /// Example
    /// ```rust,no_run
    /// let value : int32 = info.get_query("some_key").unwrap();
    /// 
    /// println!("{}", value);
    /// 
    /// ```
    pub fn get_query<T: FromStr>(&self, k : &str) -> Option<T>
    where
        <T as FromStr>::Err: std::fmt::Debug,
    {

        if let Some(querys) = &self.querys {

            if let Some(value) = querys.get(k) {

                if let Ok(parsed) = value.parse::<T>() {
    
                    return Some(parsed);
    
                }
    
            }

        }

        None
    }
    
    /// ### Get param with key
    /// 
    /// Example
    /// ```rust,no_run
    /// let value : bool = info.get_param("some_key").unwrap();
    /// 
    /// println!("{}", value);
    /// 
    /// ```
    pub fn get_param<T: FromStr>(&self, k : &str) -> Option<T>
    where
        <T as FromStr>::Err: std::fmt::Debug,
    {

        if let Some(value) = self.params.get(k) {

            if let Ok(parsed) = value.parse::<T>() {

                return Some(parsed);

            }

        }

        None
    }

}


const PARAM_SEGMENT_START: &str = "<";
const PARAM_SEGMENT_END: &str = ">";


//
// Private: Path Parser
//
pub fn parse_path(path: &str, route: &str, querys : Option<&str>) -> Option<URLInfo> {

    let p_parts: Vec<&str> = path.trim_matches('/').split('/').collect();
    let r_parts: Vec<&str> = route.trim_matches('/').split('/').collect();

    if p_parts.len() != r_parts.len() {
        return None;
    }

    let mut params = HashMap::new();

    for (path_segment, template_segment) in p_parts.iter().zip(r_parts.iter()) {

        if template_segment.starts_with(PARAM_SEGMENT_START) && template_segment.ends_with(PARAM_SEGMENT_END) {

            let key = &template_segment[1..template_segment.len() - 1];
            let val = *path_segment;

            params.insert(key.to_string(), val.to_string());

        } else if path_segment != template_segment {

            return None;

        }

    }

    Some(URLInfo { params, querys : parse_query(querys) })

}



//
// Private: Query Parser
//
fn parse_query(query_string: Option<&str>) -> Option<HashMap<String, String>> {

    let mut querys = HashMap::new();

    if let Some(query) = query_string {

        for pair in query.split('&') {

            let mut iter = pair.split('=');

            if let (Some(key), Some(value)) = (iter.next(), iter.next()) {

                querys.insert(key.to_string(), value.to_string());

            }

        }

        return Some(querys);

    }

    None

}