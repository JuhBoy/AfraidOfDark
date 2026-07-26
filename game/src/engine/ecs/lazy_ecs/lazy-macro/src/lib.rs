extern crate proc_macro;
use proc_macro::{
    Delimiter, Group, Ident, Punct, Span, TokenStream,
    TokenTree::{self},
};

#[derive(Debug)]
pub(crate) enum RTType {
    UNDEFINED,
    STRUCT,
    TRAIT,
    UNION,
}
#[derive(Debug)]
pub(crate) struct Descriptor {
    is_public: bool,
    rtt: RTType,
    main_identifier: String,
}
impl Descriptor {
    pub fn new() -> Self {
        Self {
            is_public: false,
            rtt: RTType::UNDEFINED,
            main_identifier: String::from(""),
        }
    }
}

#[proc_macro_attribute]
pub fn lazy_ecs_component(_attr: TokenStream, input: TokenStream) -> TokenStream {
    let input_tree: Vec<TokenTree> = input.into_iter().collect();

    let mut descriptor: Descriptor = Descriptor::new();
    let mut found_public: bool = false;
    let mut found_rtt = false;
    let mut found_main_ident = false;
    let mut group_found = false;

    let mut output_stream = TokenStream::new();

    for token in input_tree.iter() {
        match token {
            TokenTree::Group(group) => {
                if group_found {
                    continue;
                }

                let debug_cfg = debug_cfg_attribute();
                let debug_name = [
                    TokenTree::Ident(Ident::new("pub", Span::call_site())),
                    TokenTree::Ident(Ident::new("___debug_name", Span::call_site())),
                    TokenTree::Punct(Punct::new(':', proc_macro::Spacing::Alone)),
                    TokenTree::Ident(Ident::new("String", Span::call_site())),
                    TokenTree::Punct(Punct::new(',', proc_macro::Spacing::Alone)),
                ];
                let mut debug_stream = TokenStream::new();

                let original: Vec<TokenTree> = group.stream().into_iter().collect();
                debug_stream.extend(original);
                debug_stream.extend(debug_cfg);
                debug_stream.extend(debug_name);

                let mut new_body = TokenTree::Group(Group::new(Delimiter::Brace, debug_stream));
                new_body.set_span(group.span());
                output_stream.extend(TokenStream::from(new_body));

                group_found = true;
            }
            TokenTree::Ident(ident) => {
                output_stream.extend([token.clone()]);

                if found_main_ident {
                    continue;
                }

                let ident_str = ident.to_string();

                if !found_public && ident_str == "pub" {
                    found_public = true;
                    descriptor.is_public = true;
                    continue;
                }

                if !found_rtt {
                    let rtt_type = match ident_str.as_str() {
                        "struct" => RTType::STRUCT,
                        "trait" => RTType::TRAIT,
                        "union" => RTType::UNION,
                        _ => panic!(""),
                    };

                    found_rtt = true;
                    descriptor.rtt = rtt_type;
                    continue;
                }

                if !found_main_ident {
                    descriptor.main_identifier = ident_str;
                    found_main_ident = true;
                }
            }
            TokenTree::Punct(punct) => output_stream.extend([token.clone()]),
            TokenTree::Literal(literal) => output_stream.extend([token.clone()]),
        }
    }

    let extension = [
        TokenTree::Ident(Ident::new("impl", Span::call_site())),
        TokenTree::Ident(Ident::new(&descriptor.main_identifier, Span::call_site())),
        TokenTree::Group(Group::new(
            proc_macro::Delimiter::Brace,
            create_fns(vec![
                FnCreateDesc {
                    name: "debug_name",
                    is_pub: true,
                    is_self: true,
                    is_mut: false,
                    is_debug_only: true,
                    arguments: vec![],
                    output_arg: Option::Some(FnOutput {
                        prefix: Some('&'),
                        rtt: "str",
                    }),
                    body_stream: "&self.___debug_name".parse().unwrap(),
                },
                FnCreateDesc {
                    name: "set_debug_name",
                    is_pub: true,
                    is_self: true,
                    is_mut: true,
                    is_debug_only: true,
                    arguments: vec![FnArgument {
                        name: "name",
                        rtt: "String",
                    }],
                    output_arg: None,
                    body_stream: "self.___debug_name = name;".parse().unwrap(),
                },
                FnCreateDesc {
                    name: "new",
                    is_pub: true,
                    is_self: false,
                    is_mut: false,
                    is_debug_only: false,
                    arguments: vec![],
                    output_arg: Some(FnOutput {
                        prefix: None,
                        rtt: "Self",
                    }),
                    body_stream: "Self { 
                        #[cfg(debug_assertions)]
                        ___debug_name: String::from(\"\"), 
                    }"
                    .parse()
                    .unwrap(),
                },
            ]),
        )),
    ];
    output_stream.extend(extension);

    output_stream
}

pub(crate) struct FnArgument<'a> {
    pub name: &'a str,
    pub rtt: &'a str,
}
pub(crate) struct FnOutput<'a> {
    pub prefix: Option<char>,
    pub rtt: &'a str,
}
pub(crate) struct FnCreateDesc<'a> {
    pub name: &'a str,
    pub is_pub: bool,
    pub is_self: bool,
    pub is_mut: bool,
    pub is_debug_only: bool,
    pub arguments: Vec<FnArgument<'a>>,
    pub output_arg: Option<FnOutput<'a>>,
    pub body_stream: TokenStream,
}

fn create_arg(arg: &FnArgument) -> TokenStream {
    let mut output = TokenStream::new();

    output.extend([
        TokenTree::Ident(Ident::new(arg.name, Span::call_site())),
        TokenTree::Punct(Punct::new(':', proc_macro::Spacing::Alone)),
        TokenTree::Ident(Ident::new(arg.rtt, Span::call_site())),
        TokenTree::Punct(Punct::new(',', proc_macro::Spacing::Alone)),
    ]);

    output
}

fn create_args(args: &Vec<FnArgument>, add_self: bool, is_mut: bool) -> TokenStream {
    let mut output = TokenStream::new();

    if add_self {
        if is_mut {
            output.extend([
                TokenTree::Punct(Punct::new('&', proc_macro::Spacing::Joint)),
                TokenTree::Ident(Ident::new("mut", Span::call_site())),
                TokenTree::Ident(Ident::new("self", Span::call_site())),
                TokenTree::Punct(Punct::new(',', proc_macro::Spacing::Alone)),
            ]);
        } else {
            output.extend([
                TokenTree::Punct(Punct::new('&', proc_macro::Spacing::Joint)),
                TokenTree::Ident(Ident::new("self", Span::call_site())),
                TokenTree::Punct(Punct::new(',', proc_macro::Spacing::Alone)),
            ]);
        }
    }

    for arg in args {
        let stream = create_arg(arg);
        output.extend(stream);
    }

    output
}

fn create_fn(desc: FnCreateDesc) -> TokenStream {
    let mut fn_stream = TokenStream::new();
    let arguments = create_args(&desc.arguments, desc.is_self, desc.is_mut);
    let body = desc.body_stream;

    let mut output_tree: Vec<TokenTree> = Vec::new();

    if desc.is_pub {
        output_tree.push(TokenTree::Ident(Ident::new("pub", Span::call_site())));
    }

    output_tree.push(TokenTree::Ident(Ident::new("fn", Span::call_site())));
    output_tree.push(TokenTree::Ident(Ident::new(desc.name, Span::call_site())));
    output_tree.push(TokenTree::Group(Group::new(
        proc_macro::Delimiter::Parenthesis,
        arguments,
    )));

    if let Some(output_arg) = desc.output_arg {
        output_tree.push(TokenTree::Punct(Punct::new(
            '-',
            proc_macro::Spacing::Joint,
        )));
        output_tree.push(TokenTree::Punct(Punct::new(
            '>',
            proc_macro::Spacing::Joint,
        )));

        if let Some(prefix) = output_arg.prefix {
            output_tree.push(TokenTree::Punct(Punct::new(
                prefix,
                proc_macro::Spacing::Alone,
            )));
        }
        output_tree.push(TokenTree::Ident(Ident::new(
            output_arg.rtt,
            Span::call_site(),
        )));
    }

    output_tree.push(TokenTree::Group(Group::new(
        proc_macro::Delimiter::Brace,
        body,
    )));

    if desc.is_debug_only {
        fn_stream.extend(debug_cfg_attribute());
    }
    fn_stream.extend(output_tree);

    fn_stream
}

fn create_fns(descs: Vec<FnCreateDesc>) -> TokenStream {
    let mut output = TokenStream::new();

    for desc in descs {
        let i = create_fn(desc);
        output.extend(i);
    }

    output
}

fn debug_cfg_attribute() -> TokenStream {
    let condition = TokenStream::from(TokenTree::Ident(Ident::new(
        "debug_assertions",
        Span::call_site(),
    )));

    let cfg_call = [
        TokenTree::Ident(Ident::new("cfg", Span::call_site())),
        TokenTree::Group(Group::new(Delimiter::Parenthesis, condition)),
    ]
    .into_iter()
    .collect();

    [
        TokenTree::Punct(Punct::new('#', proc_macro::Spacing::Alone)),
        TokenTree::Group(Group::new(Delimiter::Bracket, cfg_call)),
    ]
    .into_iter()
    .collect()
}
