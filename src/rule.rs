use ca_rules::{ParseNtLife, ParseRuleError};
use std::{
    fmt::{Debug, Formatter, Result as DebugResult},
    str::FromStr,
};

struct Rule3x3 {
    rule_table: [bool; 1 << 9],
}

impl ParseNtLife for Rule3x3 {
    fn from_bs(b: Vec<u8>, s: Vec<u8>) -> Self {
        if b.contains(&0x00) {
            unimplemented!("B0 rules are not yet supported.")
        }
        let mut rule_table = [false; 1 << 9];
        b.into_iter()
            .map(|n| ((n & 0xf0) as usize) << 1 | (n & 0x0f) as usize)
            .chain(
                s.into_iter()
                    .map(|n| ((n & 0xf0) as usize) << 1 | 0x10 | (n & 0x0f) as usize),
            )
            .for_each(|n| rule_table[n] = true);
        Rule3x3 { rule_table }
    }
}

#[derive(Clone)]
pub struct Rule {
    pub rule_table: [u8; 1 << 16],
}

impl Debug for Rule {
    fn fmt(&self, f: &mut Formatter<'_>) -> DebugResult {
        f.debug_struct("Rule").finish()
    }
}

impl From<Rule3x3> for Rule {
    fn from(rule_3x3: Rule3x3) -> Self {
        let mut rule_table = [0; 1 << 16];
        rule_table.iter_mut().enumerate().for_each(|(i, n)| {
            let nw_3x3 = (i & 0xe000) >> 7 | (i & 0x0e00) >> 6 | (i & 0x00e0) >> 5;
            let ne_3x3 = (i & 0x7000) >> 6 | (i & 0x0700) >> 5 | (i & 0x0070) >> 4;
            let sw_3x3 = (i & 0x0e00) >> 3 | (i & 0x00e0) >> 2 | (i & 0x000e) >> 1;
            let se_3x3 = (i & 0x0700) >> 2 | (i & 0x0070) >> 1 | (i & 0x0007);
            *n = (rule_3x3.rule_table[nw_3x3] as u8) << 5
                | (rule_3x3.rule_table[ne_3x3] as u8) << 4
                | (rule_3x3.rule_table[sw_3x3] as u8) << 1
                | (rule_3x3.rule_table[se_3x3] as u8);
        });
        Rule { rule_table }
    }
}

impl ParseNtLife for Rule {
    fn from_bs(b: Vec<u8>, s: Vec<u8>) -> Self {
        Rule3x3::from_bs(b, s).into()
    }
}

impl FromStr for Rule {
    type Err = ParseRuleError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Rule::parse_rule(s)
    }
}

#[cfg(test)]
#[allow(clippy::unusual_byte_groupings)]
mod tests {
    use super::{ParseNtLife, Rule, Rule3x3};
    use std::error::Error;

    #[test]
    fn parse_rule_3x3() -> Result<(), Box<dyn Error>> {
        let rule = Rule3x3::parse_rule("B3/S23")?;
        assert_eq!(rule.rule_table[0b_000_000_000], false);
        assert_eq!(rule.rule_table[0b_000_010_000], false);
        assert_eq!(rule.rule_table[0b_000_000_110], false);
        assert_eq!(rule.rule_table[0b_000_010_110], true);
        assert_eq!(rule.rule_table[0b_100_000_110], true);
        assert_eq!(rule.rule_table[0b_100_010_110], true);
        assert_eq!(rule.rule_table[0b_100_100_110], false);
        assert_eq!(rule.rule_table[0b_100_110_110], false);
        Ok(())
    }

    #[test]
    fn parse_rule() -> Result<(), Box<dyn Error>> {
        let rule = Rule::parse_rule("B3/S23")?;
        assert_eq!(rule.rule_table[0b_0000_0000_0000_0000], 0b_00_00_00);
        assert_eq!(rule.rule_table[0b_0011_1110_0101_1110], 0b_00_00_00);
        assert_eq!(rule.rule_table[0b_0111_0010_0010_1001], 0b_00_00_11);
        assert_eq!(rule.rule_table[0b_0000_0111_0001_0001], 0b_01_00_00);
        assert_eq!(rule.rule_table[0b_1000_1000_1111_1100], 0b_01_00_01);
        assert_eq!(rule.rule_table[0b_0011_1110_0000_0011], 0b_11_00_00);
        Ok(())
    }
}
