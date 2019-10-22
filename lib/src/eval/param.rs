use super::Var;

pub struct Parameters(pub Vec<Var>);

macro_rules! conversion_wrapper {
    ( $( ($fn_name:ident, $fn_wrapped:ident, $result:ident) $(,)? )* ) => {
        $(
        #[inline]
        pub fn $fn_name(&self) -> Result<Vec<$result>, String> {
            Ok(self
               .0
               .iter()
               .fold(Ok(Vec::new()), |acc: Result<_, String>, x| {
                   let mut args = acc?;
                   args.push(x.$fn_wrapped()?);
                   Ok(args)
               })?)
        })*
    };
}

impl Parameters {
    #[rustfmt::skip]
    conversion_wrapper!(
        (numbers , number , f64     )
        (strings , string , String  )
        (booleans, boolean, bool    )
    );
}
