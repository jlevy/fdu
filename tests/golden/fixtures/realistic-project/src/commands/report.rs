use crate::{Index, Report, Selection, View};

pub fn build(index: &Index, selection: &Selection, views: &[View]) -> Report {
    let mut report = Report::new(index.root());
    for view in views {
        match view {
            View::Tree => report.push_tree(index.tree(selection)),
            View::Types => report.push_types(index.types(selection)),
            View::Files => report.push_files(index.files(selection)),
            View::Summary => report.push_summary(index.summary(selection)),
        }
    }
    report
}

pub fn render(report: &Report) -> String {
    let mut output = String::new();
    for section in report.sections() {
        section.write_human(&mut output);
    }
    output
}
