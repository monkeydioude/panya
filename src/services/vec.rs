use crate::db::model::FieldSort;

pub trait RemoveExisting<V: PartialEq, T: FieldSort<V>> {
    fn remove_existing(&self, from: &[T]) -> Vec<T>;
}

impl<V: PartialEq, T: FieldSort<V> + Clone> RemoveExisting<V, T> for Vec<T> {
    fn remove_existing(&self, from: &[T]) -> Vec<T> {
        self.iter()
            .filter(|&elt| {
                !from
                    .iter()
                    .any(|v| v.sort_by_value() == elt.sort_by_value())
            })
            .cloned()
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(PartialEq, Clone, Debug)]
    struct Dummy {
        field: String,
    }

    impl FieldSort<String> for Dummy {
        fn sort_by_value(&self) -> String {
            self.field.clone()
        }
    }

    #[test]
    fn test_can_remove_existing1() {
        let vec1 = vec![
            Dummy {
                field: "a".to_string(),
            },
            Dummy {
                field: "b".to_string(),
            },
        ];
        let vec2 = vec![
            Dummy {
                field: "a".to_string(),
            },
            Dummy {
                field: "c".to_string(),
            },
        ];
        let res = vec1.remove_existing(&vec2);
        assert_eq!(
            res,
            vec![Dummy {
                field: "b".to_string(),
            }]
        );
    }

    #[test]
    fn test_can_remove_existing() {
        let vec1 = vec![Dummy {
            field: "a".to_string(),
        }];
        let vec2 = vec![];
        let res = vec1.remove_existing(&vec2);
        assert_eq!(
            res,
            vec![Dummy {
                field: "a".to_string()
            }]
        );
    }
}
