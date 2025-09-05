fn main() {
    // read input from command line arguments
    let query: String = std::env::args().skip(1).collect::<Vec<_>>().join(" ");
    let parsed_query = tankseek_core::query::query_parser::parse_query(&query);
    println!("{:#?}", parsed_query);

}
