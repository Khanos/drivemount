use crate::{
    errors::Result,
    security::validation,
    system::process::{binary, CommandSpec, Runner},
};
use std::collections::HashMap;
pub fn parse(text: &str) -> HashMap<String, String> {
    text.lines()
        .filter_map(|l| l.split_once('=').map(|(k, v)| (k.into(), v.into())))
        .collect()
}
pub async fn show(r: &dyn Runner, unit: &str) -> Result<HashMap<String, String>> {
    validation::unit_name(unit)?;
    let o = r
        .run(CommandSpec::new(
            binary("systemctl")?,
            [
                "--user",
                "show",
                unit,
                "--property=ActiveState,SubState,UnitFileState,MainPID,FragmentPath,LoadState",
            ],
        ))
        .await?
        .checked()?;
    Ok(parse(&o.stdout))
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn values_may_contain_equals() {
        let p = parse("ActiveState=active\nFragmentPath=/a=b/unit\nUnitFileState=enabled\n");
        assert_eq!(p["FragmentPath"], "/a=b/unit");
    }
    #[tokio::test]
    async fn inspects_without_mutation() {
        let r = crate::system::process::fake::FakeRunner::default();
        r.push(true, "ActiveState=failed\nSubState=failed");
        assert_eq!(
            show(&r, "drive.service").await.unwrap()["ActiveState"],
            "failed"
        );
        let calls = r.calls.lock().unwrap();
        assert_eq!(calls.len(), 1);
        assert!(calls[0].args.contains(&"show".into()));
    }
}
