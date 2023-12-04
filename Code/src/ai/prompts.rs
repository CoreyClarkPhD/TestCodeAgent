use std::vec;

use crate::compiler::MappedJsonError;

use super::{Message, Role};

pub fn get_mistral_prompt(output_json: &MappedJsonError, file_contents: &str) -> Vec<Message> {
    let mut result = vec::Vec::new();

    result.push(Message {
        role: Role::System,
        content:
            "You are an extremely smart assistant that helps with fixing c++ compiler errors \n
You will be given a json output of the clang compiler and then the original contents of the file.
Please output a markdown response with the corrected source code and an explanation of what went wrong. The source code should come before the explanation. Priortize giving the source code over the explantion if you can't do both.
Print the entire corrected source code file using the ```cpp tag, then an empty line then the explanation of exactly how you fixed the error.
"
            .to_string(),
    });

    result.push(Message {
        role: Role::User,
        content:
"Compiler output: {\n  \"column\": 10,\n  \"line\": 2,\n  \"filepath\": \"./ExampleCode/bad.cpp\",\n  \"message\": \"'cout' is not a member of 'std'\",\n  \"snippet\": \"int main() {\\n    std::cout << \\\"Hello World!\\\";\\n    return 0;\\n}\\n\"\n}\nOriginal File: int main() {\n    std::cout << \"Hello World!\";\n    return 0;\n}\n\n".to_string()
    });

    result.push(Message{
        role: Role::Assistant,
        content: "
```cpp
#include <iostream>

int main() {
    std::cout << \"Hello World!\";
    return 0;
}
```

---

The error message `\"'cout' is not a member of 'std'\"` indicates that the `cout` object, which is defined in the header `<iostream>`, is not recognized in the namespace `std`. This typically occurs when the `<iostream>` header has not been included in the file, which is necessary to use `cout`.

The correction involves adding the line `#include <iostream>` at the top of the file. This includes the declaration of `cout` along with other input/output stream objects and functionalities provided by the standard library. With this fixed, the code should compile without any errors.
".to_string()
    });

    result.push(Message {
        role: Role::User,
        content: format!(
         "Here is a new program and error for you to correct. It is a different program from the previous question.\n
             Compiler output: {}\nOriginal File: {}\n",
            serde_json::to_string_pretty(output_json).expect("Pretty print jso"),
            file_contents
        ),
    });

    result
}

pub fn get_chat_gpt_prompt(output_json: &MappedJsonError, file_contents: &str) -> Vec<Message> {
    let mut result = vec::Vec::new();

    result.push(Message {
        role: Role::System,
        content:
            "You are an extremely smart assistant that helps with fixing c++ compiler errors \n
You will be given a json output of the clang compiler and then the original contents of the file.
Please output a markdown response with the corrected source code and an explanation of what went wrong.
Print the entire corrected source code file using the ```cpp tag, then an empty line then the explanation.
"
            .to_string(),
    });

    result.push(Message {
        role: Role::User,
        content: format!(
            "Compiler output: {}\nOriginal File: {}\n",
            serde_json::to_string_pretty(output_json).expect("Pretty print jso"),
            file_contents
        ),
    });

    result
}

pub fn get_mini_orca_prompt(output_json: &MappedJsonError, file_contents: &str) -> Vec<Message> {
    let mut result = vec::Vec::new();

    result.push(Message {
        role: Role::System,
        content:
            "You are an extremely smart assistant that helps with fixing c++ compiler errors \n
You will be given a json output of the clang compiler and then the original contents of the file.
Please output a markdown response with the corrected source code and an explanation of what went wrong. The source code should come before the explanation. Priortize giving the source code over the explantion if you can't do both.
Print the entire corrected source code file using the ```cpp tag, then an empty line then the explanation of exactly how you fixed the error.
            Your response should begin with \"```c++ and then a new line.
"
            .to_string(),
    });

    result.push(Message {
        role: Role::User,
        content: format!(
            "Compiler output: {}\nOriginal File: {}\n",
            serde_json::to_string_pretty(output_json).expect("Pretty print jso"),
            file_contents
        ),
    });

    result
}
