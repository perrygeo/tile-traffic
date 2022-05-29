pub fn make_url(template: String, c: usize) -> String {
    let mut url = template;
    // http://localhost:7800/osm.points/z/x/y.pbf
    // Move diagonally
    let z = 7;
    let x = 102 + c;
    let y = 53 + c;

    url = url.replace("{z}", z.to_string().as_ref());
    url = url.replace("{x}", x.to_string().as_ref());
    url = url.replace("{y}", y.to_string().as_ref());
    url
}
