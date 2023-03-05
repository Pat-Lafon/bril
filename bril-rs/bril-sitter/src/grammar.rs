#[rust_sitter::grammar("bril")]
pub mod bril_grammar {
    use rust_sitter::Spanned;
    use std::path::PathBuf;

    #[rust_sitter::language]
    #[derive(Debug)]
    pub struct ParserProgram {
        pub(crate) imports: Vec<ParserImport>,
        pub(crate) functions: Vec<ParserFunction>,
    }

    #[derive(Debug)]
    pub(crate) struct ParserImport {
        #[rust_sitter::leaf(text = "from")]
        _s: (),
        #[rust_sitter::leaf(pattern = r#""[^"]*""#, transform = |v| PathBuf::from(v.trim_matches('\"')))]
        pub(crate) path: PathBuf,
        #[rust_sitter::leaf(text = "import")]
        _t: (),
        #[rust_sitter::delimited(
            #[rust_sitter::leaf(text = ",")]
            ()
        )]
        pub(crate) functions: Vec<ParserImportedFunction>,
        #[rust_sitter::leaf(text = ";")]
        _e: (),
    }

    #[derive(Debug)]
    pub(crate) struct ParserImportedFunction {
        pub(crate) name: Func,
        pub(crate) alias: Option<Alias>,
    }

    #[derive(Debug)]
    pub(crate) struct Alias {
        #[rust_sitter::leaf(text = "as")]
        _a: (),
        pub(crate) alias: Func,
    }

    #[derive(Debug)]
    pub(crate) struct ParserFunction {
        pub(crate) name: Spanned<Func>,
        pub(crate) args: Option<ParserArgumentList>,
        pub(crate) ty: Spanned<Option<ParserOutputType>>,
        #[rust_sitter::leaf(text = "{")]
        _l: (),
        pub(crate) code: Vec<ParserCode>,
        #[rust_sitter::leaf(text = "}")]
        _r: (),
    }

    #[derive(Debug)]
    pub(crate) struct ParserArgumentList {
        #[rust_sitter::leaf(text = "(")]
        _l: (),
        #[rust_sitter::delimited(
            #[rust_sitter::leaf(text = ",")]
            ()
        )]
        pub(crate) args: Vec<ParserArgument>,
        #[rust_sitter::leaf(text = ")")]
        _r: (),
    }

    #[derive(Debug)]
    pub(crate) struct ParserArgument {
        pub(crate) name: Ident,
        #[rust_sitter::leaf(text = ":")]
        _c: (),
        pub(crate) arg_type: ParserType,
    }

    #[derive(Debug)]
    pub(crate) enum ParserCode {
        Label(Spanned<Label>, #[rust_sitter::leaf(text = ":")] Spanned<()>),
        Instruction(ParserInstruction),
    }

    #[derive(Debug)]
    pub(crate) enum ParserInstruction {
        Constant(
            Spanned<Ident>,
            Option<ParserOutputType>,
            #[rust_sitter::leaf(text = "=")] (),
            ParserConstOps,
            ParserLiteral,
            #[rust_sitter::leaf(text = ";")] Spanned<()>,
        ),
        Value(
            Spanned<Ident>,
            Option<ParserOutputType>,
            #[rust_sitter::leaf(text = "=")] (),
            Ident,
            Vec<Args>,
            #[rust_sitter::leaf(text = ";")] Spanned<()>,
        ),
        Effect(
            Spanned<Ident>,
            Vec<Args>,
            #[rust_sitter::leaf(text = ";")] Spanned<()>,
        ),
    }

    #[derive(Debug)]
    pub(crate) enum ParserConstOps {
        Const(#[rust_sitter::leaf(text = "const")] ()),
    }

    #[derive(Debug)]
    pub(crate) enum Args {
        Func(Func),
        Label(Label),
        Ident(Ident),
    }

    #[derive(Debug)]
    pub(crate) struct ParserOutputType {
        #[rust_sitter::leaf(text = ":")]
        _c: (),
        pub(crate) arg_type: Spanned<ParserType>,
    }

    #[derive(Debug)]
    pub(crate) enum ParserType {
        Primitive(Ident),
        Parameterized(
            Ident,
            #[rust_sitter::leaf(text = "<")] (),
            Box<ParserType>,
            #[rust_sitter::leaf(text = ">")] (),
        ),
    }

    #[derive(Debug)]
    pub(crate) enum ParserLiteral {
        Int(
            #[rust_sitter::leaf(pattern = r"(\+|-)?[0-9]+", transform = |v| v.parse().unwrap())]
            i64,
        ),
        Bool(
            #[rust_sitter::leaf(pattern = r"(true|false)", transform = |v| v.parse().unwrap())]
            bool,
        ),
        Float(
            // https://stackoverflow.com/questions/12643009/regular-expression-for-floating-point-numbers
            #[rust_sitter::leaf(pattern = r"(\+|-)?(((([0-9]+\.?[0-9]*)|(\.[0-9]+))(E|e)(\+|-)?[0-9]+)|(([0-9]+\.[0-9]*)|(\.[0-9]+)))", transform = |v| v.parse().unwrap())]
             f64,
        ),
    }

    #[derive(Debug)]
    pub(crate) struct Func {
        #[rust_sitter::leaf(pattern = r"@(_|%|[A-Za-z])(_|%|\.|[A-Za-z]|[0-9])*", transform = |v| v.strip_prefix("@").unwrap().to_owned())]
        pub(crate) name: String,
    }

    #[derive(Debug)]
    pub(crate) struct Label {
        #[rust_sitter::leaf(pattern = r"\.(_|%|[A-Za-z])(_|%|\.|[A-Za-z]|[0-9])*", transform = |v| v.strip_prefix(".").unwrap().to_owned())]
        pub(crate) name: String,
    }

    #[derive(Debug)]
    pub(crate) struct Ident {
        #[rust_sitter::leaf(pattern = r"(_|%|[A-Za-z])(_|%|\.|[A-Za-z]|[0-9])*", transform = |v| v.to_owned())]
        pub(crate) name: String,
    }

    #[rust_sitter::extra]
    struct Whitespace {
        #[rust_sitter::leaf(pattern = r"\s")]
        _whitespace: (),
    }

    #[rust_sitter::extra]
    struct Comment {
        #[rust_sitter::leaf(pattern = r"#[^\n\r]*[\n\r]*")]
        _comment: (),
    }
}
