//! Logic for matching and capturing route strings.
use yew::{Component, Html};

pub use yew_router_route_parser::{Captures, FromCaptures, FromCapturesError, CaptureVariant, MatcherToken};

#[cfg(feature = "regex_matcher")]
mod regex_matcher;
#[cfg(feature = "regex_matcher")]
use regex::Regex;

#[cfg(feature = "route_matcher")]
pub mod route_matcher;
#[cfg(feature = "route_matcher")]
pub use self::route_matcher::RouteMatcher;

mod custom;
pub use custom::CustomMatcher;


/// Trait that allows user-defined matchers.
pub trait MatcherProvider {
    /// Given itself and a route string, determine if the route matches by returning an Option
    /// possibly containing any sections captured by the matcher.
    fn match_route_string<'a, 'b: 'a>(&'b self, route_string: &'a str) -> Option<Captures<'a>>;
}


/// An enum that contains variants that can match a route string
#[derive(Clone, Debug)]
pub enum Matcher {

    #[cfg(feature = "route_matcher")]
    /// A matcher generated by the route macro.
    RouteMatcher(RouteMatcher),
    #[cfg(feature = "regex_matcher")]
    /// A matcher that uses a regex to match and capture values.
    RegexMatcher(Regex),
    /// A user-defined matcher.
    CustomMatcher(CustomMatcher)
}

impl PartialEq for Matcher {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Matcher::RouteMatcher(this), Matcher::RouteMatcher(other)) => this == other,
            (Matcher::RegexMatcher(this), Matcher::RegexMatcher(other)) => this.as_str() == other.as_str(),
            (Matcher::CustomMatcher(this), Matcher::CustomMatcher(other)) => this == other,
            _ => false
        }
    }
}

impl Matcher {
    /// Given itself and a route string, determine if the route matches by returning an Option
    /// possibly containing any sections captured by the matcher.
    pub fn match_route_string<'a, 'b: 'a>(&'b self, route_string: &'a str) -> Option<Captures<'a>> {
        match self {
            #[cfg(feature = "route_matcher")]
            Matcher::RouteMatcher(matcher) => {
                matcher.match_route(route_string).map(|x| x.1).ok()
            }
            #[cfg(feature = "regex_matcher")]
            Matcher::RegexMatcher(regex) => regex.match_route_string(route_string),
            Matcher::CustomMatcher(matcher) => matcher.match_route_string(route_string)
        }
    }
}

/// Render function.
pub trait RenderFn<CTX: Component>: Fn(&Captures) -> Option<Html<CTX>> {}

impl<CTX, T> RenderFn<CTX> for T
    where
        T: Fn(&Captures) -> Option<Html<CTX>>,
        CTX: Component,
{
}


