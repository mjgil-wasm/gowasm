use super::stdlib_packages;

fn package<'a>(packages: &'a [super::StdlibPackage], name: &str) -> &'a super::StdlibPackage {
    packages
        .iter()
        .find(|package| package.name == name)
        .unwrap_or_else(|| panic!("stdlib package `{name}` should be registered"))
}

#[test]
fn exposes_registered_stdlib_packages() {
    let all_packages = stdlib_packages();
    assert_eq!(all_packages.len(), 34);
    assert_eq!(
        package(all_packages, "context").functions[0].symbol,
        "Background"
    );
    assert_eq!(
        package(all_packages, "encoding/json").functions[0].symbol,
        "Marshal"
    );
    let packages = [
        package(all_packages, "builtin"),
        package(all_packages, "bytes"),
        package(all_packages, "cmp"),
        package(all_packages, "errors"),
        package(all_packages, "fmt"),
        package(all_packages, "log"),
        package(all_packages, "math"),
        package(all_packages, "os"),
        package(all_packages, "path"),
        package(all_packages, "sort"),
        package(all_packages, "maps"),
        package(all_packages, "slices"),
        package(all_packages, "strings"),
        package(all_packages, "strconv"),
        package(all_packages, "unicode"),
        package(all_packages, "math/rand"),
        package(all_packages, "math/bits"),
        package(all_packages, "unicode/utf8"),
        package(all_packages, "regexp"),
        package(all_packages, "encoding/hex"),
        package(all_packages, "crypto/sha256"),
        package(all_packages, "crypto/md5"),
        package(all_packages, "crypto/sha1"),
        package(all_packages, "crypto/sha512"),
        package(all_packages, "encoding/base64"),
        package(all_packages, "time"),
        package(all_packages, "io/fs"),
        package(all_packages, "path/filepath"),
        package(all_packages, "net/http"),
        package(all_packages, "net/url"),
        package(all_packages, "reflect"),
        package(all_packages, "sync"),
    ];
    assert_eq!(packages.len(), 32);
    assert_eq!(packages[0].name, "builtin");
    assert_eq!(packages[0].functions[0].symbol, "len");
    assert_eq!(packages[0].functions[1].symbol, "append");
    assert_eq!(packages[0].functions[2].symbol, "__range_keys");
    assert_eq!(packages[0].functions[3].symbol, "cap");
    assert_eq!(packages[0].functions[4].symbol, "__range_value");
    assert_eq!(packages[0].functions[5].symbol, "delete");
    assert_eq!(packages[0].functions[6].symbol, "__make_slice");
    assert_eq!(packages[0].functions[7].symbol, "__append_spread");
    assert_eq!(packages[0].functions[8].symbol, "min");
    assert_eq!(packages[0].functions[9].symbol, "max");
    assert_eq!(packages[0].functions[10].symbol, "clear");
    assert_eq!(packages[1].name, "bytes");
    assert_eq!(packages[1].functions[0].symbol, "Contains");
    assert_eq!(packages[1].functions[12].symbol, "SplitN");
    assert_eq!(packages[1].functions[13].symbol, "SplitAfterN");
    assert_eq!(packages[1].functions[14].symbol, "SplitAfter");
    assert_eq!(packages[1].functions[17].symbol, "Fields");
    assert_eq!(packages[1].functions[21].symbol, "ToTitle");
    assert_eq!(packages[1].functions[24].symbol, "ContainsRune");
    assert_eq!(packages[1].functions[25].symbol, "IndexAny");
    assert_eq!(packages[1].functions[26].symbol, "LastIndexAny");
    assert_eq!(packages[1].functions[27].symbol, "IndexRune");
    assert_eq!(packages[1].functions[28].symbol, "Clone");
    assert_eq!(packages[1].functions[29].symbol, "EqualFold");
    assert_eq!(packages[1].functions[30].symbol, "Trim");
    assert_eq!(packages[1].functions[31].symbol, "TrimLeft");
    assert_eq!(packages[1].functions[32].symbol, "TrimRight");
    assert_eq!(packages[1].functions[33].symbol, "IndexFunc");
    assert_eq!(packages[1].functions[34].symbol, "LastIndexFunc");
    assert_eq!(packages[1].functions[35].symbol, "TrimFunc");
    assert_eq!(packages[1].functions[36].symbol, "TrimLeftFunc");
    assert_eq!(packages[1].functions[37].symbol, "TrimRightFunc");
    assert_eq!(packages[1].functions[38].symbol, "FieldsFunc");
    assert_eq!(packages[1].functions[39].symbol, "Map");
    assert_eq!(packages[1].functions[40].symbol, "CutPrefix");
    assert_eq!(packages[1].functions[41].symbol, "CutSuffix");
    assert_eq!(packages[1].functions[42].symbol, "Cut");
    assert_eq!(packages[2].name, "cmp");
    assert_eq!(packages[2].functions[0].symbol, "Compare");
    assert_eq!(packages[2].functions[1].symbol, "Less");
    assert_eq!(packages[2].functions[2].symbol, "Or");
    assert_eq!(packages[3].name, "errors");
    assert_eq!(packages[3].functions[0].symbol, "New");
    assert_eq!(packages[3].functions[1].symbol, "Join");
    assert_eq!(packages[3].functions[2].symbol, "Unwrap");
    assert_eq!(packages[3].functions[3].symbol, "Is");
    assert_eq!(packages[3].functions[4].symbol, "As");
    assert_eq!(packages[4].name, "fmt");
    assert_eq!(packages[4].functions[0].symbol, "Println");
    assert_eq!(packages[4].functions[1].symbol, "Sprintf");
    assert_eq!(packages[4].functions[2].symbol, "Printf");
    assert_eq!(packages[4].functions[3].symbol, "Sprint");
    assert_eq!(packages[4].functions[4].symbol, "Sprintln");
    assert_eq!(packages[4].functions[5].symbol, "Errorf");
    assert_eq!(packages[5].name, "log");
    assert_eq!(packages[5].functions[0].symbol, "SetFlags");
    assert_eq!(packages[5].functions[1].symbol, "Flags");
    assert_eq!(packages[5].functions[2].symbol, "SetPrefix");
    assert_eq!(packages[5].functions[3].symbol, "Prefix");
    assert_eq!(packages[5].functions[4].symbol, "Println");
    assert_eq!(packages[5].functions[5].symbol, "Printf");
    assert_eq!(packages[5].functions[6].symbol, "Print");
    assert_eq!(packages[5].functions[7].symbol, "Fatal");
    assert_eq!(packages[5].functions[8].symbol, "Fatalf");
    assert_eq!(packages[6].name, "math");
    assert_eq!(packages[6].functions[0].symbol, "Abs");
    assert_eq!(packages[6].functions[1].symbol, "Ceil");
    assert_eq!(packages[6].functions[2].symbol, "Floor");
    assert_eq!(packages[6].functions[3].symbol, "Max");
    assert_eq!(packages[6].functions[4].symbol, "Min");
    assert_eq!(packages[6].functions[5].symbol, "Mod");
    assert_eq!(packages[6].functions[6].symbol, "Pow");
    assert_eq!(packages[6].functions[7].symbol, "Round");
    assert_eq!(packages[6].functions[8].symbol, "Sqrt");
    assert_eq!(packages[6].functions[9].symbol, "Trunc");
    assert_eq!(packages[6].functions[10].symbol, "Sin");
    assert_eq!(packages[6].functions[11].symbol, "Cos");
    assert_eq!(packages[6].functions[12].symbol, "Tan");
    assert_eq!(packages[6].functions[13].symbol, "Log");
    assert_eq!(packages[6].functions[14].symbol, "Log2");
    assert_eq!(packages[6].functions[15].symbol, "Log10");
    assert_eq!(packages[6].functions[16].symbol, "Exp");
    assert_eq!(packages[6].functions[17].symbol, "Atan2");
    assert_eq!(packages[6].functions[18].symbol, "Hypot");
    assert_eq!(packages[6].functions[19].symbol, "Inf");
    assert_eq!(packages[6].functions[20].symbol, "NaN");
    assert_eq!(packages[6].functions[21].symbol, "IsNaN");
    assert_eq!(packages[6].functions[22].symbol, "IsInf");
    assert_eq!(packages[6].functions[23].symbol, "Asin");
    assert_eq!(packages[6].functions[24].symbol, "Acos");
    assert_eq!(packages[6].functions[25].symbol, "Atan");
    assert_eq!(packages[6].functions[26].symbol, "Sinh");
    assert_eq!(packages[6].functions[27].symbol, "Cosh");
    assert_eq!(packages[6].functions[28].symbol, "Tanh");
    assert_eq!(packages[6].functions[29].symbol, "Remainder");
    assert_eq!(packages[6].functions[30].symbol, "Dim");
    assert_eq!(packages[6].functions[31].symbol, "Copysign");
    assert_eq!(packages[6].functions[32].symbol, "Signbit");
    assert_eq!(packages[6].functions[33].symbol, "Ldexp");
    assert_eq!(packages[6].functions[34].symbol, "Frexp");
    assert_eq!(packages[6].functions[35].symbol, "Expm1");
    assert_eq!(packages[6].functions[36].symbol, "Log1p");
    assert_eq!(packages[6].functions[37].symbol, "Cbrt");
    assert_eq!(packages[6].functions[38].symbol, "Float64bits");
    assert_eq!(packages[6].functions[39].symbol, "Float64frombits");
    assert_eq!(packages[6].functions[40].symbol, "Logb");
    assert_eq!(packages[6].functions[41].symbol, "Ilogb");
    assert_eq!(packages[6].functions[42].symbol, "Modf");
    assert_eq!(packages[6].constants[0].symbol, "Pi");
    assert_eq!(packages[6].constants[1].symbol, "E");
    assert_eq!(packages[6].constants[2].symbol, "Phi");
    assert_eq!(packages[6].constants[3].symbol, "Sqrt2");
    assert_eq!(packages[6].constants[4].symbol, "Ln2");
    assert_eq!(packages[6].constants[5].symbol, "Ln10");
    assert_eq!(packages[6].constants[6].symbol, "MaxFloat64");
    assert_eq!(packages[6].constants[7].symbol, "SmallestNonzeroFloat64");
    assert_eq!(packages[6].constants[8].symbol, "MaxInt");
    assert_eq!(packages[6].constants[9].symbol, "MinInt");
    assert_eq!(packages[6].constants[10].symbol, "MaxInt8");
    assert_eq!(packages[6].constants[11].symbol, "MaxInt16");
    assert_eq!(packages[6].constants[12].symbol, "MaxInt32");
    assert_eq!(packages[6].constants[13].symbol, "MaxInt64");
    assert_eq!(packages[6].constants[14].symbol, "MaxUint8");
    assert_eq!(packages[6].constants[15].symbol, "MaxUint16");
    assert_eq!(packages[6].constants[16].symbol, "MaxUint32");
    assert_eq!(packages[6].constants[17].symbol, "MinInt8");
    assert_eq!(packages[6].constants[18].symbol, "MinInt16");
    assert_eq!(packages[6].constants[19].symbol, "MinInt32");
    assert_eq!(packages[6].constants[20].symbol, "MinInt64");
    assert_eq!(packages[6].constants[21].symbol, "MaxFloat32");
    assert_eq!(packages[6].constants[22].symbol, "SmallestNonzeroFloat32");
    assert_eq!(packages[7].name, "os");
    assert_eq!(packages[7].functions[0].symbol, "Exit");
    assert_eq!(packages[7].functions[8].symbol, "WriteFile");
    assert_eq!(packages[7].functions[9].symbol, "Environ");
    assert_eq!(packages[7].functions[10].symbol, "ExpandEnv");
    assert_eq!(packages[7].functions[11].symbol, "Expand");
    assert_eq!(packages[7].functions[12].symbol, "ReadDir");
    assert_eq!(packages[7].functions[13].symbol, "Stat");
    assert_eq!(packages[7].functions[14].symbol, "Lstat");
    assert_eq!(packages[7].functions[15].symbol, "MkdirAll");
    assert_eq!(packages[7].functions[16].symbol, "RemoveAll");
    assert_eq!(packages[7].functions[17].symbol, "Getwd");
    assert_eq!(packages[7].functions[18].symbol, "IsExist");
    assert_eq!(packages[7].functions[19].symbol, "IsNotExist");
    assert_eq!(packages[7].functions[20].symbol, "IsPathSeparator");
    assert_eq!(packages[7].functions[25].symbol, "TempDir");
    assert_eq!(packages[7].functions[26].symbol, "UserHomeDir");
    assert_eq!(packages[7].functions[27].symbol, "UserCacheDir");
    assert_eq!(packages[7].functions[28].symbol, "UserConfigDir");
    assert_eq!(packages[7].functions[29].symbol, "Hostname");
    assert_eq!(packages[7].functions[30].symbol, "Executable");
    assert_eq!(packages[7].functions[31].symbol, "Getuid");
    assert_eq!(packages[7].functions[32].symbol, "Geteuid");
    assert_eq!(packages[7].functions[33].symbol, "Getgid");
    assert_eq!(packages[7].functions[34].symbol, "Getegid");
    assert_eq!(packages[7].functions[35].symbol, "Getpid");
    assert_eq!(packages[7].functions[36].symbol, "Getppid");
    assert_eq!(packages[7].functions[37].symbol, "Getpagesize");
    assert_eq!(packages[7].functions[38].symbol, "Getgroups");
    assert_eq!(packages[7].constants[0].symbol, "PathSeparator");
    assert_eq!(packages[8].name, "path");
    assert_eq!(packages[8].functions[0].symbol, "Base");
    assert_eq!(packages[8].functions[1].symbol, "Clean");
    assert_eq!(packages[8].functions[2].symbol, "Dir");
    assert_eq!(packages[8].functions[3].symbol, "Ext");
    assert_eq!(packages[8].functions[4].symbol, "IsAbs");
    assert_eq!(packages[8].functions[5].symbol, "Split");
    assert_eq!(packages[8].functions[6].symbol, "Join");
    assert_eq!(packages[8].functions[7].symbol, "Match");
    assert_eq!(packages[9].name, "sort");
    assert_eq!(packages[9].functions[0].symbol, "IntsAreSorted");
    assert_eq!(packages[9].functions[1].symbol, "StringsAreSorted");
    assert_eq!(packages[9].functions[2].symbol, "SearchInts");
    assert_eq!(packages[9].functions[3].symbol, "SearchStrings");
    assert_eq!(packages[9].functions[4].symbol, "Ints");
    assert_eq!(packages[9].functions[5].symbol, "Strings");
    assert_eq!(packages[9].functions[6].symbol, "Float64s");
    assert_eq!(packages[9].functions[7].symbol, "Float64sAreSorted");
    assert_eq!(packages[9].functions[8].symbol, "SearchFloat64s");
    assert_eq!(packages[9].functions[9].symbol, "Slice");
    assert_eq!(packages[9].functions[10].symbol, "SliceIsSorted");
    assert_eq!(packages[9].functions[11].symbol, "SliceStable");
    assert_eq!(packages[9].functions[12].symbol, "Search");
    assert_eq!(packages[10].name, "maps");
    assert_eq!(packages[10].functions[0].symbol, "Keys");
    assert_eq!(packages[10].functions[1].symbol, "Values");
    assert_eq!(packages[10].functions[2].symbol, "Equal");
    assert_eq!(packages[10].functions[3].symbol, "EqualFunc");
    assert_eq!(packages[10].functions[4].symbol, "Clone");
    assert_eq!(packages[10].functions[5].symbol, "Copy");
    assert_eq!(packages[10].functions[6].symbol, "DeleteFunc");
    assert_eq!(packages[11].name, "slices");
    assert_eq!(packages[11].functions[0].symbol, "Contains");
    assert_eq!(packages[11].functions[1].symbol, "ContainsFunc");
    assert_eq!(packages[11].functions[2].symbol, "Index");
    assert_eq!(packages[11].functions[3].symbol, "IndexFunc");
    assert_eq!(packages[11].functions[4].symbol, "SortFunc");
    assert_eq!(packages[11].functions[5].symbol, "SortStableFunc");
    assert_eq!(packages[11].functions[6].symbol, "Compact");
    assert_eq!(packages[11].functions[7].symbol, "CompactFunc");
    assert_eq!(packages[11].functions[8].symbol, "Reverse");
    assert_eq!(packages[11].functions[9].symbol, "Equal");
    assert_eq!(packages[12].name, "strings");
    assert_eq!(packages[12].functions[0].symbol, "Contains");
    assert_eq!(packages[12].functions[1].symbol, "HasPrefix");
    assert_eq!(packages[12].functions[2].symbol, "HasSuffix");
    assert_eq!(packages[12].functions[3].symbol, "TrimSpace");
    assert_eq!(packages[12].functions[4].symbol, "ToUpper");
    assert_eq!(packages[12].functions[5].symbol, "ToLower");
    assert_eq!(packages[12].functions[6].symbol, "ToTitle");
    assert_eq!(packages[12].functions[7].symbol, "Count");
    assert_eq!(packages[12].functions[8].symbol, "Repeat");
    assert_eq!(packages[12].functions[9].symbol, "Split");
    assert_eq!(packages[12].functions[10].symbol, "Join");
    assert_eq!(packages[12].functions[11].symbol, "NewReplacer");
    assert_eq!(packages[12].functions[12].symbol, "ReplaceAll");
    assert_eq!(packages[12].functions[13].symbol, "Fields");
    assert_eq!(packages[12].functions[14].symbol, "Index");
    assert_eq!(packages[12].functions[15].symbol, "TrimPrefix");
    assert_eq!(packages[12].functions[16].symbol, "TrimSuffix");
    assert_eq!(packages[12].functions[17].symbol, "LastIndex");
    assert_eq!(packages[12].functions[18].symbol, "TrimLeft");
    assert_eq!(packages[12].functions[19].symbol, "TrimRight");
    assert_eq!(packages[12].functions[20].symbol, "Trim");
    assert_eq!(packages[12].functions[21].symbol, "ContainsAny");
    assert_eq!(packages[12].functions[22].symbol, "IndexAny");
    assert_eq!(packages[12].functions[23].symbol, "LastIndexAny");
    assert_eq!(packages[12].functions[24].symbol, "Clone");
    assert_eq!(packages[12].functions[25].symbol, "ContainsRune");
    assert_eq!(packages[12].functions[26].symbol, "IndexRune");
    assert_eq!(packages[12].functions[27].symbol, "Compare");
    assert_eq!(packages[12].functions[28].symbol, "Replace");
    assert_eq!(packages[12].functions[29].symbol, "IndexByte");
    assert_eq!(packages[12].functions[30].symbol, "LastIndexByte");
    assert_eq!(packages[12].functions[31].symbol, "CutPrefix");
    assert_eq!(packages[12].functions[32].symbol, "CutSuffix");
    assert_eq!(packages[12].functions[33].symbol, "Cut");
    assert_eq!(packages[12].functions[34].symbol, "EqualFold");
    assert_eq!(packages[12].functions[35].symbol, "SplitN");
    assert_eq!(packages[12].functions[36].symbol, "SplitAfterN");
    assert_eq!(packages[12].functions[37].symbol, "SplitAfter");
    assert_eq!(packages[12].functions[38].symbol, "Map");
    assert_eq!(packages[12].functions[39].symbol, "IndexFunc");
    assert_eq!(packages[12].functions[40].symbol, "LastIndexFunc");
    assert_eq!(packages[12].functions[41].symbol, "TrimFunc");
    assert_eq!(packages[12].functions[42].symbol, "TrimLeftFunc");
    assert_eq!(packages[12].functions[43].symbol, "TrimRightFunc");
    assert_eq!(packages[12].functions[44].symbol, "FieldsFunc");
    assert_eq!(packages[13].name, "strconv");
    assert_eq!(packages[13].functions[0].symbol, "Itoa");
    assert_eq!(packages[13].functions[1].symbol, "FormatBool");
    assert_eq!(packages[13].functions[2].symbol, "Quote");
    assert_eq!(packages[13].functions[3].symbol, "CanBackquote");
    assert_eq!(packages[13].functions[4].symbol, "FormatInt");
    assert_eq!(packages[13].functions[5].symbol, "FormatUint");
    assert_eq!(packages[13].functions[6].symbol, "QuoteToASCII");
    assert_eq!(packages[13].functions[7].symbol, "QuoteRuneToASCII");
    assert_eq!(packages[13].functions[8].symbol, "QuoteRune");
    assert_eq!(packages[13].functions[9].symbol, "Atoi");
    assert_eq!(packages[13].functions[10].symbol, "ParseBool");
    assert_eq!(packages[13].functions[11].symbol, "ParseInt");
    assert_eq!(packages[13].functions[12].symbol, "Unquote");
    assert_eq!(packages[13].functions[13].symbol, "UnquoteChar");
    assert_eq!(packages[13].functions[14].symbol, "FormatFloat");
    assert_eq!(packages[13].functions[15].symbol, "ParseFloat");
    assert_eq!(packages[13].functions[16].symbol, "ParseUint");
    assert!(packages[13].constants.is_empty());
    assert_eq!(packages[14].name, "unicode");
    assert_eq!(packages[14].functions[0].symbol, "IsDigit");
    assert_eq!(packages[14].functions[1].symbol, "IsLetter");
    assert_eq!(packages[14].functions[2].symbol, "IsSpace");
    assert_eq!(packages[14].functions[3].symbol, "IsUpper");
    assert_eq!(packages[14].functions[4].symbol, "IsLower");
    assert_eq!(packages[14].functions[5].symbol, "IsNumber");
    assert_eq!(packages[14].functions[6].symbol, "IsPrint");
    assert_eq!(packages[14].functions[7].symbol, "IsGraphic");
    assert_eq!(packages[14].functions[8].symbol, "IsPunct");
    assert_eq!(packages[14].functions[9].symbol, "IsSymbol");
    assert_eq!(packages[14].functions[10].symbol, "IsMark");
    assert_eq!(packages[14].functions[11].symbol, "IsTitle");
    assert_eq!(packages[14].functions[12].symbol, "IsControl");
    assert_eq!(packages[14].functions[13].symbol, "ToUpper");
    assert_eq!(packages[14].functions[14].symbol, "ToLower");
    assert_eq!(packages[14].functions[15].symbol, "ToTitle");
    assert_eq!(packages[14].functions[16].symbol, "To");
    assert_eq!(packages[14].functions[17].symbol, "SimpleFold");
    assert_eq!(packages[14].constants[0].symbol, "Version");
    assert_eq!(packages[14].constants[1].symbol, "MaxRune");
    assert_eq!(packages[14].constants[2].symbol, "ReplacementChar");
    assert_eq!(packages[14].constants[3].symbol, "MaxASCII");
    assert_eq!(packages[14].constants[4].symbol, "MaxLatin1");
    assert_eq!(packages[14].constants[5].symbol, "UpperCase");
    assert_eq!(packages[14].constants[6].symbol, "LowerCase");
    assert_eq!(packages[14].constants[7].symbol, "TitleCase");
    assert_eq!(packages[15].name, "math/rand");
    assert_eq!(packages[15].functions.len(), 8);
    assert_eq!(packages[15].functions[0].symbol, "Intn");
    assert_eq!(packages[15].functions[1].symbol, "Float64");
    assert_eq!(packages[15].functions[2].symbol, "Int");
    assert_eq!(packages[15].functions[3].symbol, "Seed");
    assert_eq!(packages[15].functions[4].symbol, "Int63");
    assert_eq!(packages[15].functions[5].symbol, "Int63n");
    assert_eq!(packages[15].functions[6].symbol, "Int31");
    assert_eq!(packages[15].functions[7].symbol, "Int31n");
    assert!(packages[15].constants.is_empty());
    assert_eq!(packages[24].name, "encoding/base64");
    assert_eq!(
        packages[24].functions[0].symbol,
        "StdEncodingEncodeToString"
    );
    assert_eq!(packages[24].functions[1].symbol, "StdEncodingDecodeString");
    assert_eq!(
        packages[24].functions[2].symbol,
        "URLEncodingEncodeToString"
    );
    assert_eq!(packages[24].functions[3].symbol, "URLEncodingDecodeString");
    assert_eq!(
        packages[24].functions[4].symbol,
        "RawStdEncodingEncodeToString"
    );
    assert_eq!(
        packages[24].functions[5].symbol,
        "RawStdEncodingDecodeString"
    );
    assert_eq!(
        packages[24].functions[6].symbol,
        "RawURLEncodingEncodeToString"
    );
    assert_eq!(
        packages[24].functions[7].symbol,
        "RawURLEncodingDecodeString"
    );
    assert!(packages[24].constants.is_empty());
    assert_eq!(packages[25].name, "time");
    assert_eq!(packages[25].functions[0].symbol, "Now");
    assert_eq!(packages[25].functions[1].symbol, "Unix");
    assert_eq!(packages[25].functions[2].symbol, "UnixMilli");
    assert_eq!(packages[25].functions[3].symbol, "UnixMicro");
    assert_eq!(packages[25].functions[4].symbol, "Since");
    assert_eq!(packages[25].functions[5].symbol, "Until");
    assert_eq!(packages[25].functions[6].symbol, "Parse");
    assert_eq!(packages[25].constants[0].symbol, "Nanosecond");
    assert_eq!(packages[25].constants[1].symbol, "Microsecond");
    assert_eq!(packages[25].constants[2].symbol, "Millisecond");
    assert_eq!(packages[25].constants[3].symbol, "Second");
    assert_eq!(packages[25].constants[4].symbol, "Minute");
    assert_eq!(packages[25].constants[5].symbol, "Hour");
    assert_eq!(packages[25].constants[6].symbol, "DateTime");
    assert_eq!(packages[25].constants[7].symbol, "ANSIC");
    assert_eq!(packages[25].constants[8].symbol, "RFC850");
    assert_eq!(packages[25].constants[9].symbol, "RFC1123");
    assert_eq!(packages[25].constants[10].symbol, "RFC1123Z");
    assert_eq!(packages[25].constants[11].symbol, "RFC3339");
    assert_eq!(packages[26].name, "io/fs");
    assert_eq!(packages[26].functions[0].symbol, "ValidPath");
    assert_eq!(packages[26].functions[1].symbol, "ReadFile");
    assert_eq!(packages[26].functions[2].symbol, "Stat");
    assert_eq!(packages[26].functions[3].symbol, "Sub");
    assert_eq!(packages[26].functions[4].symbol, "Glob");
    assert_eq!(packages[26].functions[5].symbol, "ReadDir");
    assert_eq!(packages[26].functions[6].symbol, "WalkDir");
    assert_eq!(packages[26].functions[7].symbol, "FileInfoToDirEntry");
    assert_eq!(packages[26].functions[8].symbol, "FormatDirEntry");
    assert_eq!(packages[26].functions[9].symbol, "FormatFileInfo");
    assert_eq!(packages[26].constants[0].symbol, "ModeDir");
    assert_eq!(packages[26].constants[1].symbol, "ModeAppend");
    assert_eq!(packages[26].constants[4].symbol, "ModeSymlink");
    assert_eq!(packages[26].constants[13].symbol, "ModeType");
    assert_eq!(packages[26].constants[14].symbol, "ModePerm");
    assert_eq!(packages[27].name, "path/filepath");
    assert_eq!(packages[27].functions[0].symbol, "Base");
    assert_eq!(packages[27].functions[1].symbol, "Clean");
    assert_eq!(packages[27].functions[2].symbol, "Dir");
    assert_eq!(packages[27].functions[3].symbol, "Ext");
    assert_eq!(packages[27].functions[4].symbol, "IsAbs");
    assert_eq!(packages[27].functions[5].symbol, "Split");
    assert_eq!(packages[27].functions[6].symbol, "Join");
    assert_eq!(packages[27].functions[7].symbol, "Match");
    assert_eq!(packages[27].functions[8].symbol, "ToSlash");
    assert_eq!(packages[27].functions[9].symbol, "FromSlash");
    assert_eq!(packages[27].functions[10].symbol, "SplitList");
    assert_eq!(packages[27].functions[11].symbol, "VolumeName");
    assert_eq!(packages[27].functions[12].symbol, "Rel");
    assert_eq!(packages[27].functions[13].symbol, "IsLocal");
    assert_eq!(packages[27].functions[14].symbol, "Localize");
    assert_eq!(packages[27].functions[15].symbol, "Glob");
    assert_eq!(packages[27].functions[16].symbol, "WalkDir");
    assert_eq!(packages[27].functions[17].symbol, "Walk");
    assert_eq!(packages[27].functions[18].symbol, "Abs");
    assert_eq!(packages[28].name, "net/http");
    assert_eq!(packages[28].functions[0].symbol, "CanonicalHeaderKey");
    assert_eq!(packages[28].functions[1].symbol, "StatusText");
    assert_eq!(packages[28].functions[2].symbol, "ParseHTTPVersion");
    assert_eq!(packages[28].functions[3].symbol, "DetectContentType");
    assert_eq!(packages[28].functions[4].symbol, "ParseTime");
    assert_eq!(packages[28].functions[5].symbol, "NewRequest");
    assert_eq!(packages[28].functions[6].symbol, "NewRequestWithContext");
    assert_eq!(packages[28].constants[0].symbol, "MethodGet");
    assert_eq!(packages[28].constants[4].symbol, "MethodPatch");
    assert_eq!(packages[28].constants[8].symbol, "MethodTrace");
    assert_eq!(packages[28].constants[9].symbol, "TimeFormat");
    assert_eq!(packages[28].constants[10].symbol, "TrailerPrefix");
    assert_eq!(packages[28].constants[11].symbol, "DefaultMaxHeaderBytes");
    assert!(packages[28]
        .constants
        .iter()
        .any(|entry| entry.symbol == "StatusOK"));
    assert!(packages[28]
        .constants
        .iter()
        .any(|entry| entry.symbol == "StatusNetworkAuthenticationRequired"));
    assert_eq!(packages[29].name, "net/url");
    assert_eq!(packages[29].functions[0].symbol, "Parse");
    assert!(packages[29].constants.is_empty());
    assert_eq!(packages[30].name, "reflect");
    assert_eq!(packages[30].functions[0].symbol, "TypeOf");
    assert_eq!(packages[30].functions[1].symbol, "ValueOf");
    assert_eq!(packages[30].constants[0].symbol, "Invalid");
    assert_eq!(packages[30].constants[9].symbol, "Ptr");
    assert_eq!(packages[30].constants[12].symbol, "Struct");
    assert_eq!(packages[31].name, "sync");
    assert!(packages[31].functions.is_empty());
    assert!(packages[31].constants.is_empty());
}
