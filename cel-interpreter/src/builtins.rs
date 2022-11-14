use chrono::NaiveDate;
use uuid::Uuid;

use std::rc::Rc;

use super::value::*;
use crate::error::*;

pub(crate) fn date(args: Vec<CelValue>) -> Result<CelValue, CelError> {
    let s: Rc<String> = assert_arg(args.get(0))?;
    Ok(CelValue::Date(NaiveDate::parse_from_str(&s, "%Y-%m-%d")?))
}

pub(crate) fn uuid(args: Vec<CelValue>) -> Result<CelValue, CelError> {
    let s: Rc<String> = assert_arg(args.get(0))?;
    Ok(CelValue::Uuid(
        s.parse()
            .map_err(|e| CelError::UuidError(format!("{e:?}")))?,
    ))
}

fn assert_arg<'a, T: TryFrom<&'a CelValue, Error = CelError>>(
    arg: Option<&'a CelValue>,
) -> Result<T, CelError> {
    if let Some(v) = arg {
        T::try_from(v)
    } else {
        Err(CelError::MissingArgument)
    }
}