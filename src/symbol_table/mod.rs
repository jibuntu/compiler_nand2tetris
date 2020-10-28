// SymbolTableモジュール
// 仕様は267page

use std::collections::HashMap;


#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Kind {
    Static, // スタティック変数
    Field, // フィールド変数
    Arg, // 関数の引数
    Var, // 関数内の変数
}

const KIND_LIST: [Kind;4] = [Kind::Static, Kind::Field, Kind::Arg, Kind::Var];

pub struct SymbolTable {
    // HashMap<名前, (型, 属性, インデックス)>
    class_scope: HashMap<String, (String, Kind, usize)>,
    subroutine_scope: HashMap<String, (String, Kind, usize)>,
    counter: HashMap<Kind, usize>
}

impl SymbolTable {
    pub fn new() -> SymbolTable {
        let mut counter: HashMap<Kind, usize> = HashMap::new();
        for k in &KIND_LIST {
            counter.insert(*k, 0);
        }

        SymbolTable {
            class_scope: HashMap::new(),
            subroutine_scope: HashMap::new(),
            counter
        }
    }

    // subroutine_scopeのテーブルを初期化する
    pub fn start_subroutine(&mut self) {
        self.subroutine_scope = HashMap::new();
        // カウンタも初期化する
        *self.counter.get_mut(&Kind::Arg).unwrap() = 0;
        *self.counter.get_mut(&Kind::Var).unwrap() = 0;
    }

    // symbol tableに値を追加する
    pub fn define(&mut self, name: String, ty: String, kind: Kind) {
        let index = *self.counter.get(&kind).unwrap();
        match kind {
            Kind::Static | Kind::Field => {
                self.class_scope.insert(name, (ty, kind, index));
            },
            Kind::Arg | Kind::Var => {
                self.subroutine_scope.insert(name, (ty, kind, index));
            }
        }

        *self.counter.get_mut(&kind).unwrap() += 1;
    }

    // 引数で与えられた属性について、それが現在のスコープで定義されている数を返す
    pub fn var_count(&self, kind: Kind) -> usize {
        *self.counter.get(&kind).unwrap()
    }

    // 引数で与えられた名前の識別子を現在のスコープで探し、その属性を返す。
    // 見つからないときはNoneを返す
    fn kind_of(&self, name: &str) -> Option<Kind> {
        if let Some((_, k, _)) = self.subroutine_scope.get(name) {
            return Some(*k)
        }

        if let Some((_, k, _)) = self.class_scope.get(name) {
            return Some(*k)
        }

        None
    }

    // 引数で与えられた名前の識別子を現在のスコープで探し、その型を返す
    pub fn type_of(&self, name: &str) -> Option<&str> {
        if let Some((t, _, _)) = self.subroutine_scope.get(name) {
            return Some(t)
        }

        if let Some((t, _, _)) = self.class_scope.get(name) {
            return Some(t)
        }

        None
    }

    // 引数で与えられた名前の識別子を現在のスコープで探し、そのインデックスを返す
    pub fn index_of(&self, name: &str) -> Option<usize> {
        if let Some((_, _, i)) = self.subroutine_scope.get(name) {
            return Some(*i)
        }

        if let Some((_, _, i)) = self.class_scope.get(name) {
            return Some(*i)
        }

        None
    }
}


#[cfg(test)]
mod test_symbol_table {
    use super::SymbolTable;
    use super::Kind;

    #[test]
    fn test_new() {
        let mut st = SymbolTable::new();
    }

    #[test]
    fn test_define() {
        let mut st = SymbolTable::new();
        st.define("name".to_string(), "type".to_string(), Kind::Var);
        st.define("namae".to_string(), "kata".to_string(), Kind::Field);
    }

    #[test]
    fn test_var_count() {
        let mut st = SymbolTable::new();
        assert_eq!(st.var_count(Kind::Static), 0);
        
        st.define("name".to_string(), "type".to_string(), Kind::Static);
        assert_eq!(st.var_count(Kind::Static), 1);

        st.define("namae".to_string(), "kata".to_string(), Kind::Static);
        assert_eq!(st.var_count(Kind::Static), 2);

        st.define("nombre".to_string(), "tipos".to_string(), Kind::Var);
        assert_eq!(st.var_count(Kind::Var), 1);
    }

    #[test]
    fn test_type_of() {
        let mut st = SymbolTable::new();
        assert_eq!(st.type_of("name"), None);

        st.define("name".to_string(), "type".to_string(), Kind::Static);
        assert_eq!(st.type_of("name"), Some("type"));

        st.define("namae".to_string(), "kata".to_string(), Kind::Arg);
        assert_eq!(st.type_of("name"), Some("type"));
    }

    #[test]
    fn test_kind_of() {
        let mut st = SymbolTable::new();
        assert_eq!(st.kind_of("name"), None);

        st.define("name".to_string(), "type".to_string(), Kind::Static);
        assert_eq!(st.kind_of("name"), Some(Kind::Static));

        st.define("namae".to_string(), "kata".to_string(), Kind::Arg);
        assert_eq!(st.kind_of("namae"), Some(Kind::Arg));
    }

    #[test]
    fn test_index_of() {
        let mut st = SymbolTable::new();
        assert_eq!(st.index_of("name"), None);

        st.define("name".to_string(), "type".to_string(), Kind::Static);
        assert_eq!(st.index_of("name"), Some(0));

        st.define("namae".to_string(), "kata".to_string(), Kind::Static);
        assert_eq!(st.index_of("namae"), Some(1));
    }

    #[test]
    fn test_start_subroutine() {
        let mut st = SymbolTable::new();

        st.define("name".to_string(), "type".to_string(), Kind::Static);
        assert_eq!(st.index_of("name"), Some(0));

        st.start_subroutine();
        assert_eq!(st.index_of("name"), Some(0));

        st.define("namae".to_string(), "kata".to_string(), Kind::Var);
        assert_eq!(st.index_of("namae"), Some(0));
        assert_eq!(st.kind_of("namae"), Some(Kind::Var));

        st.start_subroutine();
        assert_eq!(st.index_of("namae"), None);
        assert_eq!(st.kind_of("namae"), None);
    }
}
