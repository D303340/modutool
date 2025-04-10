use slint::ModelRc;
use slint::SharedString;
use slint::VecModel;

pub fn _vec_to_model_rc<T: Clone + 'static>(data: Vec<T>) -> ModelRc<T> {
    let vec_model = VecModel::from(data);
    return ModelRc::new(vec_model)
}

pub fn _string_to_model_rc(string: String) -> ModelRc<SharedString> {
    let mut v: Vec<slint::SharedString> = Vec::new();

    for c in string.chars() {
        v.push(slint::SharedString::from(c.to_string()));
    }
    
    return _vec_to_model_rc(v);
}