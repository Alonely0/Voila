pub macro println_on_debug {
    ($($args:tt)*) => {
        if std::env::var("DEBUG").unwrap_or_else(|_| { "false".to_string() }) == "true" ||
           env!("CARGO_PKG_VERSION").ends_with("dev") {
               eprintln!("DEBUG-MSG [{f}:{l}] {msg}\n", f = file!(), l = line!(), msg = format!{ $($args)* })
        }
    }
}
#[macro_export]
macro_rules! mod_use {
    { $(use $mod:ident;)+ } => { $(
            mod $mod;
            pub use $mod::*;
        )+
    };
}

#[macro_export]
macro_rules! no_spec {
    ($name:literal, $value:expr, $var_spec:expr) => {{
        if $var_spec.is_some() {
            Err(ParseErrorKind::VarHasNoSpec($name))
        } else {
            Ok($value)
        }
    }};
}

#[macro_export]
macro_rules! spec {
    ($spec:literal, $type:tt, $ctor:expr, $var_spec:expr) => {
        $var_spec
            .ok_or(ParseErrorKind::VarNeedsSpec {
                var_name: $spec,
                options: &$type::OPTS,
            })
            .and_then(|var| {
                $type::detect(var).ok_or(ParseErrorKind::InvalidSpecifier {
                    variable: $spec,
                    options: &$type::OPTS,
                })
            })
            .map($ctor)
    };
}

#[macro_export]
macro_rules! i {
    { $x:expr } => {
    $x.as_ref()
    .unwrap_or(&Vec::new())
    .par_iter()
    }
}
