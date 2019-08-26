fn main() {
    let prs = prs::fetch().unwrap();

    let json = serde_json::to_string(&prs).unwrap();
    println!("{}", json);
    // for pr in prs {
    //     let text = format!(
    //         "Title:     {:?}\n\
    //         Body:      {:?}\n\
    //         HTML URL:  {:?}\n\
    //         State:     {:?}\n\
    //         Closed at: {:?}\n\
    //         ---------\n",
    //         pr.title, pr.body, pr.html_url,
    //         pr.state, pr.closed_at
    //     );
    //     println!("{}", text);
    // }
}
