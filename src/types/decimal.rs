use crate::Error;
use bigdecimal::{BigDecimal, FromPrimitive};
use rbs::Value;
use serde::Deserializer;
use std::cmp::Ordering;
use std::fmt::{Debug, Display, Formatter};
use std::ops::{Add, AddAssign, Deref, DerefMut, Div, Mul, MulAssign, Neg, Rem, Sub, SubAssign};
use std::str::FromStr;

#[derive(serde::Serialize, Clone, Eq, PartialEq, Hash)]
#[serde(rename = "Decimal")]
pub struct Decimal(pub BigDecimal);

impl Decimal {
    pub fn new(arg: &str) -> Result<Self, Error> {
        Decimal::from_str(arg)
    }

    pub fn from_f64(arg: f64) -> Option<Decimal> {
        match BigDecimal::from_f64(arg) {
            None => { None }
            Some(v) => {
                Some(Decimal::from(v))
            }
        }
    }

    pub fn from_f32(arg: f32) -> Option<Decimal> {
        match BigDecimal::from_f32(arg) {
            None => { None }
            Some(v) => {
                Some(Decimal::from(v))
            }
        }
    }

    ///Return a new Decimal object equivalent to self,
    /// with internal scaling set to the number specified.
    /// If the new_scale is lower than the current value (indicating a larger power of 10),
    /// digits will be dropped (as precision is lower)
    pub fn with_scale(self, arg: i64) -> Self {
        Decimal(self.0.with_scale(arg))
    }

    ///Return a new Decimal object with precision set to new value
    /// let n: Decimal = "129.41675".parse().unwrap();
    ///
    /// assert_eq!(n.with_prec(2),  "130".parse().unwrap());
    ///
    /// let n_p12 = n.with_prec(12);
    /// let (i, scale) = n_p12.as_bigint_and_exponent();
    /// assert_eq!(n_p12, "129.416750000".parse().unwrap());
    /// assert_eq!(i, 129416750000_u64.into());
    /// assert_eq!(scale, 9);
    ///
    pub fn with_prec(self, arg: u64) -> Self {
        Decimal(self.0.with_prec(arg))
    }
}

impl<'de> serde::Deserialize<'de> for Decimal {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        use serde::de::Error;
        match Value::deserialize(deserializer)?.into_string() {
            None => Err(D::Error::custom("warn type decode Decimal")),
            Some(v) => Ok(Decimal::from_str(&v).map_err(|e| D::Error::custom(e.to_string())))?,
        }
    }
}

impl Display for Decimal {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Debug for Decimal {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "Decimal({})", self.0)
    }
}

impl From<BigDecimal> for Decimal {
    fn from(value: BigDecimal) -> Self {
        Self(value)
    }
}
impl From<Decimal> for Value {
    fn from(arg: Decimal) -> Self {
        Value::Ext("Decimal", Box::new(Value::String(arg.0.to_string())))
    }
}


impl From<i32> for Decimal {
    fn from(arg: i32) -> Self {
        Self::from(BigDecimal::from(arg))
    }
}

impl From<u32> for Decimal {
    fn from(arg: u32) -> Self {
        Self::from(BigDecimal::from(arg))
    }
}

impl From<i64> for Decimal {
    fn from(arg: i64) -> Self {
        Self::from(BigDecimal::from(arg))
    }
}

impl From<u64> for Decimal {
    fn from(arg: u64) -> Self {
        Self::from(BigDecimal::from(arg))
    }
}

impl FromStr for Decimal {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Decimal(BigDecimal::from_str(&s).map_err(|e| Error::from(e.to_string()))?))
    }
}


impl Deref for Decimal {
    type Target = BigDecimal;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Decimal {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl Default for Decimal {
    fn default() -> Self {
        Decimal(BigDecimal::from(0))
    }
}

impl Add for Decimal {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Decimal(self.0.add(rhs.0))
    }
}

impl Sub for Decimal {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Decimal(self.0.sub(rhs.0))
    }
}

impl Mul for Decimal {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        Decimal(self.0.mul(rhs.0))
    }
}

impl Div for Decimal {
    type Output = Self;

    fn div(self, rhs: Self) -> Self::Output {
        Decimal(self.0.div(rhs.0))
    }
}

impl PartialOrd for Decimal {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.0.partial_cmp(&other.0)
    }
}

impl Rem for Decimal {
    type Output = Decimal;

    fn rem(self, other: Decimal) -> Decimal {
        Decimal(self.0.rem(other.0))
    }
}

impl Neg for Decimal {
    type Output = Decimal;

    fn neg(self) -> Self::Output {
        Decimal(self.0.neg())
    }
}

impl AddAssign for Decimal {
    fn add_assign(&mut self, rhs: Self) {
        self.0.add_assign(rhs.0)
    }
}

impl MulAssign for Decimal {
    fn mul_assign(&mut self, rhs: Self) {
        self.0.mul_assign(rhs.0)
    }
}

impl SubAssign for Decimal {
    fn sub_assign(&mut self, rhs: Self) {
        self.0.sub_assign(rhs.0)
    }
}

#[cfg(test)]
mod test {
    use crate::decimal::Decimal;
    use rbs::{from_value, to_value};
    use std::str::FromStr;

    #[test]
    fn test_add() {
        let v1 = Decimal::from_str("1").unwrap();
        let v2 = Decimal::from_str("1.1").unwrap();
        let v = v1 + v2;
        assert_eq!(v, Decimal::from_str("2.1").unwrap());
    }

    #[test]
    fn test_sub() {
        let v1 = Decimal::new("1").unwrap();
        let v2 = Decimal::new("1.1").unwrap();
        let v = v1 - v2;
        assert_eq!(v, Decimal::new("-0.1").unwrap());
    }

    #[test]
    fn test_mul() {
        let v1 = Decimal::new("1").unwrap();
        let v2 = Decimal::new("1.1").unwrap();
        let v = v1 * v2;
        assert_eq!(v, Decimal::new("1.1").unwrap());
    }

    #[test]
    fn test_div() {
        let v1 = Decimal::new("1").unwrap();
        let v2 = Decimal::new("1.1").unwrap();
        let v = v2 / v1;
        assert_eq!(v, Decimal::new("1.1").unwrap());
    }

    #[test]
    fn test_ser() {
        let v1 = Decimal::from_str("1").unwrap();
        let rv: Decimal = from_value(to_value!(v1)).unwrap();
        assert_eq!(rv, Decimal::from_str("1").unwrap());
    }

    #[test]
    fn test_ser2() {
        let v1 = Decimal::from_str("1").unwrap();
        let rv: Decimal = serde_json::from_value(serde_json::to_value(v1).unwrap()).unwrap();
        assert_eq!(rv, Decimal::from_str("1").unwrap());
    }

    #[test]
    fn test_ser3() {
        let v1 = to_value!("1.111");
        let rv: Decimal = rbs::from_value(v1.clone()).unwrap();
        assert_eq!(rv, Decimal::from_str("1.111").unwrap());
    }

    #[test]
    fn test_with_scale() {
        let v1 = Decimal::new("1.123456").unwrap();
        let v = v1.with_scale(2);
        println!("{}", v.to_string());
        assert_eq!(v.to_string(),"1.12");
    }

    #[test]
    fn test_with_prec() {
        let v1 = Decimal::new("1.123456").unwrap();
        let v = v1.with_prec(2);
        println!("{}", v.to_string());
        assert_eq!(v.to_string(),"1.1");
    }

    #[test]
    fn test_parse() {
        let v1 = "1.123456".parse::<Decimal>().unwrap();
        assert_eq!(v1.to_string(),"1.123456");
    }
}
