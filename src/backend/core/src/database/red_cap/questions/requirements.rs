use ahash::HashMap;
use rhai::{CustomType, TypeBuilder};

use super::{QuestionAnswerMCB, QuestionAnswerRadio, QuestionType};

#[derive(Debug, Clone, CustomType)]
#[rhai_type(extra = Self::build_extra)]
struct QuestionsScriptCtx {
    questions: HashMap<String, QuestionScriptData>,
}
impl QuestionsScriptCtx {
    pub fn get(&mut self, key: &str) -> QuestionScriptData {
        self.questions.get(key).cloned().unwrap_or_default()
    }
    fn build_extra(builder: &mut TypeBuilder<Self>) {
        builder
            .with_name("QuestionsScriptCtx")
            .with_fn("get", Self::get);
    }
}

#[derive(Debug, Clone, PartialEq, CustomType)]
#[rhai_type(extra = Self::build_extra)]
pub struct QuestionScriptData {
    record_type: QuestionType,
    pub value_text: Option<String>,
    pub value_number: Option<i32>,
    pub value_float: Option<f32>,
    pub value_boolean: Option<bool>,
    pub value_radio: Option<QuestionAnswerRadio>,
    pub options: Vec<QuestionAnswerMCB>,
}
impl Default for QuestionScriptData {
    fn default() -> Self {
        Self {
            record_type: QuestionType::Text,
            value_text: Default::default(),
            value_number: Default::default(),
            value_float: Default::default(),
            value_boolean: Default::default(),
            value_radio: Default::default(),
            options: Default::default(),
        }
    }
}
impl PartialEq<bool> for QuestionScriptData {
    fn eq(&self, other: &bool) -> bool {
        self.value_boolean == Some(*other)
    }
}
impl QuestionScriptData {
    fn eq_str(&mut self, other: &str) -> bool {
        self.value_text.as_deref() == Some(other)
    }
    fn eq_bool(&mut self, other: bool) -> bool {
        self.value_boolean == Some(other)
    }
    fn eq_number(&mut self, other: i64) -> bool {
        self.value_number == Some(other as i32)
    }
    fn eq_float(&mut self, other: f32) -> bool {
        self.value_float == Some(other)
    }
    fn contains(&mut self, key: &str) -> bool {
        match self.record_type {
            QuestionType::MultiCheckBox => {
                return self
                    .options
                    .iter()
                    .any(|x| x.option_string_id.as_deref() == Some(key))
            }
            QuestionType::Radio => {
                if let Some(radio) = &self.value_radio {
                    return radio.option_name.as_deref() == Some(key);
                }
            }
            _ => {}
        }
        false
    }
    fn contains_id(&mut self, id: i64) -> bool {
        let id = id as i32;
        match self.record_type {
            QuestionType::MultiCheckBox => return self.options.iter().any(|x| x.option_id == id),
            QuestionType::Radio => {
                if let Some(radio) = &self.value_radio {
                    return radio.option_id == id;
                }
            }
            _ => {}
        }
        false
    }
    fn build_extra(builder: &mut TypeBuilder<Self>) {
        builder
            .with_name("QuestionScriptData")
            .with_fn("contains", Self::contains)
            .with_fn("contains", Self::contains_id)
            .with_fn("eq", Self::eq_str)
            .with_fn("eq", Self::eq_bool)
            .with_fn("eq", Self::eq_number)
            .with_fn("eq", Self::eq_float);
    }
}
#[cfg(test)]
mod tests {

    use ahash::{HashMap, HashMapExt};
    use rhai::{Engine, Scope};

    use crate::database::red_cap::questions::requirements::QuestionsScriptCtx;

    use super::QuestionScriptData;
    #[test]
    pub fn test() -> anyhow::Result<()> {
        let mut questions = HashMap::new();
        questions.insert(
            "text".to_string(),
            QuestionScriptData {
                value_text: Some("test".to_string()),
                ..Default::default()
            },
        );
        questions.insert(
            "number".to_string(),
            QuestionScriptData {
                value_number: Some(10),
                ..Default::default()
            },
        );
        questions.insert(
            "float".to_string(),
            QuestionScriptData {
                value_float: Some(10.0),
                ..Default::default()
            },
        );
        questions.insert(
            "boolean".to_string(),
            QuestionScriptData {
                value_boolean: Some(true),
                ..Default::default()
            },
        );
        questions.insert(
            "multi".to_owned(),
            QuestionScriptData {
                record_type: super::QuestionType::MultiCheckBox,
                options: vec![super::QuestionAnswerMCB {
                    option_id: 1,
                    option_string_id: Some("test".to_owned()),
                    option_name: "test".to_owned(),
                }],
                ..Default::default()
            },
        );
        let mut engine = Engine::new();
        let _ = engine.register_custom_operator("==", 100);
        engine.register_fn("==", |x: &mut QuestionScriptData, y: bool| x.eq_bool(y));
        engine.register_fn("==", |x: &mut QuestionScriptData, y: &str| x.eq_str(y));
        engine.register_fn("==", |x: &mut QuestionScriptData, y: i64| x.eq_number(y));
        engine.register_fn("==", |x: &mut QuestionScriptData, y: f32| x.eq_float(y));

        engine.build_type::<QuestionsScriptCtx>();
        engine.build_type::<QuestionScriptData>();
        engine
            .gen_fn_signatures(false)
            .into_iter()
            .for_each(|func| println!("{func}"));
        let mut scope = Scope::new();
        let ctx = super::QuestionsScriptCtx { questions };
        scope.push_constant("questions", ctx);
        {
            let ast = engine.compile_with_scope(&scope, r#"questions.get("boolean") == true"#)?;
            let value: bool = engine.eval_ast_with_scope(&mut scope, &ast).unwrap();
            assert!(value, "Boolean should be true");
        }
        {
            let ast = engine.compile_with_scope(&scope, r#"questions.get("text") == "test""#)?;
            let value: bool = engine.eval_ast_with_scope(&mut scope, &ast).unwrap();
            assert!(value, "Text should be test");
        }
        {
            let ast = engine.compile_with_scope(&scope, r#"questions.get("number") == 10"#)?;
            let value: bool = engine.eval_ast_with_scope(&mut scope, &ast).unwrap();
            assert!(value, "Number should be 10");
        }
        {
            let ast = engine.compile_with_scope(
                &scope,
                r#"questions.get("multi").contains("test") && questions.get("multi").contains(1)"#,
            )?;
            let value: bool = engine.eval_ast_with_scope(&mut scope, &ast).unwrap();
            assert!(value, "multi should contain test and 1");
        }
        Ok(())
    }
}
