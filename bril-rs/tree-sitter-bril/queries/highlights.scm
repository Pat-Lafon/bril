; highlights.scm

(
    (ParserLiteral) @constant.builtin
    (#match? @constant.builtin "true|false")
)
(ParserLiteral) @constant
(
    (Func) @function.builtin
    (#match? @function.builtin "main")
)
(Func) @function
(ParserConstOps_Const) @keyword
(
    (Ident) @operator
    (#match? @operator "add|sub|mul|div|id|lt|gt|eq|ge|le|print|eq|call|ret|br|jmp")
)

"," @punctuation.delimeter

(ParserType) @type

(Ident) @variable
(ParserArgument (Ident)) @variable.parameter

(Comment) @comment
(Label) @tag
