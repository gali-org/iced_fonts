use std::collections::HashMap;

use proc_macro::TokenStream;
use proc_macro2::{Ident, Span};
use quote::quote;
use syn::{
    LitInt, LitStr, Token,
    parse::{Parse, ParseStream},
    parse_macro_input,
};
use ttf_parser::Face;

struct MacroInput {
    /// e.g. `"fonts/bootstrap-icons-new.ttf"`
    font_path: LitStr,
    /// e.g. `bootstrap`
    module_name: Ident,
    /// e.g. `"BOOTSTRAP_FONT"`
    font_name: Ident,
    /// e.g. `basic`
    advanced_shaping: LitStr,
    /// e.g. `https://icons.getbootstrap.com/icons`
    doc_link: Option<LitStr>,
}

impl Parse for MacroInput {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        let font_path = input.parse()?;
        let _: Token![,] = input.parse()?;
        let module_name = input.parse()?;
        let _: Token![,] = input.parse()?;
        let font_name = input.parse()?;
        let _: Token![,] = input.parse()?;
        let advanced_shaping = input.parse()?;

        // It is good-mannered to accept an optional trailing comma
        let _: Option<Token![,]> = input.parse()?;
        let doc_link = input.parse()?;
        let _: Option<Token![,]> = input.parse()?;

        Ok(Self {
            font_path,
            module_name,
            font_name,
            advanced_shaping,
            doc_link,
        })
    }
}

/// Generates a module with functions that create text widgets.
/// 1. Parameter &str font path
/// 2. Parameter literal name for the module the macro creates.
/// 3. Parameter literal name of the font created one line above.
/// 4. Optional parameter &str of where documenation exists for this font.
#[proc_macro]
pub fn generate_icon_functions(input: TokenStream) -> TokenStream {
    let MacroInput {
        font_path,
        module_name,
        font_name,
        advanced_shaping,
        doc_link,
    } = parse_macro_input!(input as MacroInput);

    let font_path_str = font_path.value();
    let font_data = std::fs::read(&font_path_str).expect("Failed to read font file");
    let face = Face::parse(&font_data, 0).expect("Failed to parse font");

    let mut all_codepoints: Vec<char> = Vec::new();
    if let Some(unicode_subtable) = face
        .tables()
        .cmap
        .unwrap()
        .subtables
        .into_iter()
        .find(|s| s.is_unicode())
    {
        unicode_subtable.codepoints(|c| {
            use std::convert::TryFrom;
            if let Ok(u) = char::try_from(c) {
                all_codepoints.push(u);
            }
        });
    }

    let mut functions = proc_macro2::TokenStream::new();
    let mut duplicates: HashMap<String, u32> = HashMap::new();
    let mut count = 0;

    #[cfg(feature = "_generate_demo")]
    let mut demo_counter = 0;
    #[cfg(feature = "_generate_demo")]
    let mut demo_rows = 0;
    #[cfg(feature = "_generate_demo")]
    println!("row![");
    'outer: for c in all_codepoints {
        if let Some(glyph_id) = face.glyph_index(c) {
            let raw_name = face.glyph_name(glyph_id).unwrap_or("unnamed");

            // We need to rename some common characters.
            let mut processed_name = raw_name
                .replace("-", "_")
                .replace('0', "zero")
                .replace('1', "one")
                .replace('2', "two")
                .replace('3', "three")
                .replace('4', "four")
                .replace('5', "five")
                .replace('6', "six")
                .replace('7', "seven")
                .replace('8', "eight")
                .replace('9', "nine");

            // Material font edge case
            if processed_name.as_str() == "_" {
                processed_name = String::from("underscore");
            }

            // In case we have illegals. There are cases where most fonts have a .null icon that
            // doesn't do anything. So we can safely filter it out with the rest
            for c in processed_name.chars() {
                match c {
                    '+' | '-' | '*' | '/' | '@' | '!' | '#' | '$' | '%' | '^' | '&' | '(' | ')'
                    | '=' | '~' | '`' | ';' | ':' | '"' | '\'' | ',' | '<' | '>' | '?' | '.'
                    | ' ' | '[' | ']' | '{' | '}' | '|' | '\\' => continue 'outer,
                    _ => {}
                }
            }

            // Check for duplicates
            match duplicates.get(&processed_name) {
                Some(amount) => {
                    duplicates.insert(processed_name.clone(), *amount + 1);
                    // We don't care about repeats. Even though we should :(
                    continue 'outer;
                }
                None => {
                    duplicates.insert(processed_name.clone(), 1);
                }
            }

            #[cfg(feature = "_generate_demo")]
            if demo_rows < 18 {
                if demo_counter == 27 {
                    demo_counter = 0;
                    demo_rows += 1;

                    println!("{}(),", processed_name);
                    println!("]");
                    println!(".padding(12)");
                    println!(".spacing(20)");
                    println!(".width(Length::Fill)");
                    println!(".align_y(Center),");
                    println!("row![");
                } else {
                    demo_counter += 1;
                    println!("{}(),", processed_name);
                }
            }
            let fn_name = Ident::new_raw(&processed_name, Span::call_site());

            let doc = match doc_link {
                Some(ref location) => format!(
                    " Returns an [`iced`] [`iced_widget::Text`] widget of the [{} {}]({}/{}) icon.",
                    c,
                    processed_name,
                    location.value(),
                    raw_name,
                ),
                None => format!(
                    " Returns an [`iced`] [`iced_widget::Text`] widget of the {} {} icon.",
                    c, processed_name
                ),
            };

            let shaping = match advanced_shaping.value().as_str() {
                "basic" => {
                    quote! { shaping(text::Shaping::Basic) }
                }
                "advanced" => {
                    quote! { shaping(text::Shaping::Advanced) }
                }
                _ => panic!(
                    "Shaping either needs to be basic or advanced, if you are unsure use advanced."
                ),
            };

            functions.extend(quote! {
                #[doc = #doc]
                #[must_use]
                pub fn #fn_name<'a, Theme: Catalog + 'a, Renderer: text::Renderer<Font = Font>>() -> Text<'a, Theme, Renderer> {
                    use iced_widget::text;
                    text(#c).font(#font_name).#shaping
                }
            });

            count += 1;
        }
    }

    #[cfg(feature = "_generate_demo")]
    println!("We have {} icons", count);

    let count_lit = LitInt::new(&count.to_string(), Span::call_site());
    TokenStream::from(quote! {
        #[doc = "A machine generated module of font icons that create text widgets."]
        pub mod #module_name {
            use iced_widget::core::text;
            use iced_widget::core::Font;
            use iced_widget::text::Text;
            use iced_widget::text::Catalog;
            use crate::#font_name;

            /// The amount of icons in the font.
            pub const COUNT: usize = #count_lit;

            #functions
        }
    })
}
