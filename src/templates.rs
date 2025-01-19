pub mod binary {
    pub const MAIN: &'static str = "#include <stdio.h>

int main(int argc, char *argv[]) {
    printf(\"Hello, World\\n\");

    return 0;
}
";
    pub const MANIFEST: &'static str = "[package]
name = \"$name\"
version = \"0.1.0\"
edition = \"2025.1\"
";
}

pub mod library {
    pub const LIB_SOURCE: &'static str =
        "/** ----------------------------------------------------------------------------
 *  @file $name.c
 *  @brief
 *
 *  @author    John Doe <john.doe@example.com>
 *  @version   v1.0
 *  @date      $date
 *  @copyright Copyright (c)
 *  ----------------------------------------------------------------------------*/
#include \"$name/$name.h\"

int $name_sum(int a, int b) {
    return a + b;
}
";

    pub const LIB_HEADER: &'static str =
        "/** ----------------------------------------------------------------------------
 *  @file $name.h
 *  @brief
 *
 *  @author    John Doe <john.doe@example.com>
 *  @version   v1.0
 *  @date      $date
 *  @copyright Copyright (c)
 *  ----------------------------------------------------------------------------*/
#ifndef $nameup_H
#define $nameup_H

int $name_sum(int a, int b);

#endif /* $nameup_H */
";

    pub const MANIFEST: &'static str = "[package]
name = \"$name\"
version = \"0.1.0\"
edition = \"2025.1\"
type = \"lib\"
";
}
