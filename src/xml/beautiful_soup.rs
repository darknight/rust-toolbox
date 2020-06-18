use std::collections::HashMap;

struct BeautifulSoup;

impl BeautifulSoup {

    ///
    /// soup = BeautifulSoup(html_doc) => soup = BeautifulSoup::new(html_doc)
    ///
    pub fn new(soup: &str) -> BeautifulSoup {
        BeautifulSoup
    }

    ///
    /// soup.prettify()
    ///
    pub fn prettify(&self) -> String {
        String::new()
    }

    ///
    /// soup.title => soup.tag("title")
    ///
    pub fn tag(&self, tag_name: &str) -> Tag {
        unimplemented!()
    }

    ///
    /// soup.find(id="link3") => soup.find("id", "link3")
    ///
    pub fn find(&self, attr_name: &str, attr_value: &str) -> Vec<Tag> { //TODO: return Iter
        unimplemented!()
    }

    ///
    /// soup.find_all("a") => soup.find_all("a")
    ///
    pub fn find_all(&self, tag_name: &str) -> Vec<Tag> { // TODO: return Iter
        unimplemented!()
    }

    ///
    /// soup.get_text() => soup.get_text()
    ///
    pub fn get_text(&self) -> String {
        unimplemented!()
    }

}

struct Tag {
    name: String,

}

impl Tag {
    ///
    /// soup.title.name => tag.name
    ///
    pub fn name(&self) -> String {
        unimplemented!()
    }

    pub fn name_mut(&mut self, new_name: &str) {

    }

    ///
    /// soup.title.string => tag.content
    ///
    pub fn content(&self) -> String {
        unimplemented!()
    }

    ///
    /// soup.title.parent.name => tag.parent.name
    ///
    pub fn parent<'a>(&self) -> &'a Tag {
        unimplemented!()
    }

    pub fn attrs(&self) -> HashMap<String, String> {
        unimplemented!()
    }

    ///
    /// soup.p['class'] => tag.attr("class")
    ///
    pub fn attr(&self, attr_name: &str) -> Vec<String> {
        unimplemented!()
    }

    pub fn attr_mut(&mut self, attr_name: &str, attr_value: &str) {

    }

    pub fn attr_remove(&mut self, attr_name: &str) {

    }

    ///
    /// soup.p.get('class') => tag.get("class")
    ///
    pub fn get(&self, attr_name: &str) -> Option<String> {
        unimplemented!()
    }
}

impl ToString for Tag {
    fn to_string(&self) -> String {
        unimplemented!()
    }
}