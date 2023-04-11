use super::*;

#[test]
fn test_mock_simple() {
    let json = r#"
    {
        "0": 10,
        "1": true,
        "2": 2.5,
        "3": "Hello",
        "4": [],
        "5": ["Hello"],
        "6": {"a": 10}
    }
"#;
    assert_eq!(
        generate(MockAccumulator::begin(), "MyType", json).unwrap(),
        r#"ty:MyType
num:0:Int
bool:1
num:2:Float
str:3
arr:4:Null
arr:5:String
ty:_0
num:a:Int
popty
obj:6:_0
popty
"#
    );
}

#[test]
fn test_mock_nested() {
    let json = r#"
    {
        "0": {
            "1": {
                "a": 1.1
            }
        },
        "outer_array": [
            {
                "num": 10, 
                "array": [
                    {
                        "a": 10,
                        "b": 12
                    }
                ]
            },
            {
                "array": [
                    {
                        "a": 10,
                        "b": 12
                    }
                ],
                "num": 15
            }
        ]
    }
"#;
    assert_eq!(
        generate(MockAccumulator::begin(), "MyType", json).unwrap(),
        r#"ty:MyType
ty:_0
num:a:Float
popty
ty:_1
obj:1:_0
popty
obj:0:_1
ty:_2
num:a:Int
num:b:Int
popty
ty:_3
num:num:Int
arr:array:Object("_2")
popty
arr:outer_array:Object("_3")
popty
"#
    );
}

#[test]
fn test_mock_error() {
    let case = r#"
    { "a": [{ "a": "10" }, { "a": "10", "b": 5 }] }
"#;
    assert_eq!(
        generate(MockAccumulator::begin(), "MyType", case),
        Err(Error::DifferingArrayType)
    );

    let case = r#"0"#;
    assert_eq!(
        generate(MockAccumulator::begin(), "MyType", case),
        Err(Error::ExpectedObject)
    );
}
