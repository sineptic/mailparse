use super::*;

#[test]
fn parse_basic_header() {
    let (parsed, _) = parse_header(b"Key: Value").unwrap();
    assert_eq!(parsed.key, b"Key");
    assert_eq!(parsed.get_key(), "Key");
    assert_eq!(parsed.get_key_ref(), "Key");
    assert_eq!(parsed.value, b"Value");
    assert_eq!(parsed.get_value(), "Value");
    assert_eq!(parsed.get_value_raw(), "Value".as_bytes());

    let (parsed, _) = parse_header(b"Key :  Value ").unwrap();
    assert_eq!(parsed.key, b"Key ");
    assert_eq!(parsed.value, b"Value ");
    assert_eq!(parsed.get_value(), "Value ");
    assert_eq!(parsed.get_value_raw(), "Value ".as_bytes());

    let (parsed, _) = parse_header(b"Key:").unwrap();
    assert_eq!(parsed.key, b"Key");
    assert_eq!(parsed.value, b"");

    let (parsed, _) = parse_header(b":\n").unwrap();
    assert_eq!(parsed.key, b"");
    assert_eq!(parsed.value, b"");

    let (parsed, _) = parse_header(b"Key:Multi-line\n value").unwrap();
    assert_eq!(parsed.key, b"Key");
    assert_eq!(parsed.value, b"Multi-line\n value");
    assert_eq!(parsed.get_value(), "Multi-line value");
    assert_eq!(parsed.get_value_raw(), "Multi-line\n value".as_bytes());

    let (parsed, _) = parse_header(b"Key:  Multi\n  line\n value\n").unwrap();
    assert_eq!(parsed.key, b"Key");
    assert_eq!(parsed.value, b"Multi\n  line\n value");
    assert_eq!(parsed.get_value(), "Multi line value");
    assert_eq!(parsed.get_value_raw(), "Multi\n  line\n value".as_bytes());

    let (parsed, _) = parse_header(b"Key: One\nKey2: Two").unwrap();
    assert_eq!(parsed.key, b"Key");
    assert_eq!(parsed.value, b"One");

    let (parsed, _) = parse_header(b"Key: One\n\tOverhang").unwrap();
    assert_eq!(parsed.key, b"Key");
    assert_eq!(parsed.value, b"One\n\tOverhang");
    assert_eq!(parsed.get_value(), "One Overhang");
    assert_eq!(parsed.get_value_raw(), "One\n\tOverhang".as_bytes());

    let (parsed, _) = parse_header(b"SPAM: VIAGRA \xAE").unwrap();
    assert_eq!(parsed.key, b"SPAM");
    assert_eq!(parsed.value, b"VIAGRA \xAE");
    assert_eq!(parsed.get_value(), "VIAGRA \u{ae}");
    assert_eq!(parsed.get_value_raw(), b"VIAGRA \xAE");

    parse_header(b" Leading: Space").unwrap_err();

    let (parsed, _) = parse_header(b"Just a string").unwrap();
    assert_eq!(parsed.key, b"Just a string");
    assert_eq!(parsed.value, b"");
    assert_eq!(parsed.get_value(), "");
    assert_eq!(parsed.get_value_raw(), b"");

    let (parsed, _) = parse_header(b"Key\nBroken: Value").unwrap();
    assert_eq!(parsed.key, b"Key");
    assert_eq!(parsed.value, b"");
    assert_eq!(parsed.get_value(), "");
    assert_eq!(parsed.get_value_raw(), b"");

    let (parsed, _) = parse_header(b"Key: With CRLF\r\n").unwrap();
    assert_eq!(parsed.key, b"Key");
    assert_eq!(parsed.value, b"With CRLF");
    assert_eq!(parsed.get_value(), "With CRLF");
    assert_eq!(parsed.get_value_raw(), b"With CRLF");

    let (parsed, _) = parse_header(b"Key: With spurious CRs\r\r\r\n").unwrap();
    assert_eq!(parsed.value, b"With spurious CRs");
    assert_eq!(parsed.get_value(), "With spurious CRs");
    assert_eq!(parsed.get_value_raw(), b"With spurious CRs");

    let (parsed, _) = parse_header(b"Key: With \r mixed CR\r\n").unwrap();
    assert_eq!(parsed.value, b"With \r mixed CR");
    assert_eq!(parsed.get_value(), "With \r mixed CR");
    assert_eq!(parsed.get_value_raw(), b"With \r mixed CR");

    let (parsed, _) = parse_header(b"Key:\r\n Value after linebreak").unwrap();
    assert_eq!(parsed.value, b"\r\n Value after linebreak");
    assert_eq!(parsed.get_value(), " Value after linebreak");
    assert_eq!(parsed.get_value_raw(), b"\r\n Value after linebreak");
}

#[test]
fn parse_encoded_headers() {
    let (parsed, _) = parse_header(b"Subject: =?iso-8859-1?Q?=A1Hola,_se=F1or!?=").unwrap();
    assert_eq!(parsed.get_key(), "Subject");
    assert_eq!(parsed.get_key_ref(), "Subject");
    assert_eq!(parsed.get_value(), "\u{a1}Hola, se\u{f1}or!");
    assert_eq!(
        parsed.get_value_raw(),
        "=?iso-8859-1?Q?=A1Hola,_se=F1or!?=".as_bytes()
    );

    let (parsed, _) = parse_header(
        b"Subject: =?iso-8859-1?Q?=A1Hola,?=\n \
                                        =?iso-8859-1?Q?_se=F1or!?=",
    )
    .unwrap();
    assert_eq!(parsed.get_key(), "Subject");
    assert_eq!(parsed.get_key_ref(), "Subject");
    assert_eq!(parsed.get_value(), "\u{a1}Hola, se\u{f1}or!");
    assert_eq!(
        parsed.get_value_raw(),
        "=?iso-8859-1?Q?=A1Hola,?=\n \
                                          =?iso-8859-1?Q?_se=F1or!?="
            .as_bytes()
    );

    let (parsed, _) = parse_header(b"Euro: =?utf-8?Q?=E2=82=AC?=").unwrap();
    assert_eq!(parsed.get_key(), "Euro");
    assert_eq!(parsed.get_key_ref(), "Euro");
    assert_eq!(parsed.get_value(), "\u{20ac}");
    assert_eq!(parsed.get_value_raw(), "=?utf-8?Q?=E2=82=AC?=".as_bytes());

    let (parsed, _) = parse_header(b"HelloWorld: =?utf-8?B?aGVsbG8gd29ybGQ=?=").unwrap();
    assert_eq!(parsed.get_value(), "hello world");
    assert_eq!(
        parsed.get_value_raw(),
        "=?utf-8?B?aGVsbG8gd29ybGQ=?=".as_bytes()
    );

    let (parsed, _) = parse_header(b"Empty: =?utf-8?Q??=").unwrap();
    assert_eq!(parsed.get_value(), "");
    assert_eq!(parsed.get_value_raw(), "=?utf-8?Q??=".as_bytes());

    let (parsed, _) = parse_header(b"Incomplete: =?").unwrap();
    assert_eq!(parsed.get_value(), "=?");
    assert_eq!(parsed.get_value_raw(), "=?".as_bytes());

    let (parsed, _) = parse_header(b"BadEncoding: =?garbage?Q??=").unwrap();
    assert_eq!(parsed.get_value(), "=?garbage?Q??=");
    assert_eq!(parsed.get_value_raw(), "=?garbage?Q??=".as_bytes());

    let (parsed, _) = parse_header(b"Invalid: =?utf-8?Q?=E2=AC?=").unwrap();
    assert_eq!(parsed.get_value(), "\u{fffd}");

    let (parsed, _) = parse_header(b"LineBreak: =?utf-8?Q?=E2=82\n =AC?=").unwrap();
    assert_eq!(parsed.get_value(), "=?utf-8?Q?=E2=82 =AC?=");

    let (parsed, _) = parse_header(b"NotSeparateWord: hello=?utf-8?Q?world?=").unwrap();
    assert_eq!(parsed.get_value(), "hello=?utf-8?Q?world?=");

    let (parsed, _) = parse_header(b"NotSeparateWord2: =?utf-8?Q?hello?=world").unwrap();
    assert_eq!(parsed.get_value(), "=?utf-8?Q?hello?=world");

    let (parsed, _) = parse_header(b"Key: \"=?utf-8?Q?value?=\"").unwrap();
    assert_eq!(parsed.get_value(), "\"value\"");

    let (parsed, _) = parse_header(
        b"Subject: =?utf-8?q?=5BOntario_Builder=5D_Understanding_home_shopping_=E2=80=93_a_q?=\n \
                                        =?utf-8?q?uick_survey?=",
    )
    .unwrap();
    assert_eq!(parsed.get_key(), "Subject");
    assert_eq!(parsed.get_key_ref(), "Subject");
    assert_eq!(
        parsed.get_value(),
        "[Ontario Builder] Understanding home shopping \u{2013} a quick survey"
    );

    let (parsed, _) = parse_header(
        b"Subject: =?utf-8?q?=5BOntario_Builder=5D?= non-qp words\n \
             and the subject continues",
    )
    .unwrap();
    assert_eq!(
        parsed.get_value(),
        "[Ontario Builder] non-qp words and the subject continues"
    );

    let (parsed, _) = parse_header(
        b"Subject: =?utf-8?q?=5BOntario_Builder=5D?= \n \
             and the subject continues",
    )
    .unwrap();
    assert_eq!(
        parsed.get_value(),
        "[Ontario Builder]  and the subject continues"
    );
    assert_eq!(
        parsed.get_value_raw(),
        "=?utf-8?q?=5BOntario_Builder=5D?= \n \
               and the subject continues"
            .as_bytes()
    );

    let (parsed, _) = parse_header(b"Subject: =?ISO-2022-JP?B?GyRCRnwbKEI=?=\n\t=?ISO-2022-JP?B?GyRCS1wbKEI=?=\n\t=?ISO-2022-JP?B?GyRCOGwbKEI=?=")
        .unwrap();
    assert_eq!(parsed.get_key(), "Subject");
    assert_eq!(parsed.get_key_ref(), "Subject");
    assert_eq!(parsed.get_key_raw(), "Subject".as_bytes());
    assert_eq!(parsed.get_value(), "\u{65E5}\u{672C}\u{8A9E}");
    assert_eq!(parsed.get_value_raw(), "=?ISO-2022-JP?B?GyRCRnwbKEI=?=\n\t=?ISO-2022-JP?B?GyRCS1wbKEI=?=\n\t=?ISO-2022-JP?B?GyRCOGwbKEI=?=".as_bytes());

    let (parsed, _) = parse_header(b"Subject: =?ISO-2022-JP?Q?=1B\x24\x42\x46\x7C=1B\x28\x42?=\n\t=?ISO-2022-JP?Q?=1B\x24\x42\x4B\x5C=1B\x28\x42?=\n\t=?ISO-2022-JP?Q?=1B\x24\x42\x38\x6C=1B\x28\x42?=")
        .unwrap();
    assert_eq!(parsed.get_key(), "Subject");
    assert_eq!(parsed.get_key_ref(), "Subject");
    assert_eq!(parsed.get_key_raw(), "Subject".as_bytes());
    assert_eq!(parsed.get_value(), "\u{65E5}\u{672C}\u{8A9E}");
    assert_eq!(parsed.get_value_raw(), "=?ISO-2022-JP?Q?=1B\x24\x42\x46\x7C=1B\x28\x42?=\n\t=?ISO-2022-JP?Q?=1B\x24\x42\x4B\x5C=1B\x28\x42?=\n\t=?ISO-2022-JP?Q?=1B\x24\x42\x38\x6C=1B\x28\x42?=".as_bytes());

    let (parsed, _) = parse_header(b"Subject: =?UTF-7?Q?+JgM-?=").unwrap();
    assert_eq!(parsed.get_key(), "Subject");
    assert_eq!(parsed.get_key_ref(), "Subject");
    assert_eq!(parsed.get_key_raw(), "Subject".as_bytes());
    assert_eq!(parsed.get_value(), "\u{2603}");
    assert_eq!(parsed.get_value_raw(), b"=?UTF-7?Q?+JgM-?=");

    let (parsed, _) =
        parse_header(b"Content-Type: image/jpeg; name=\"=?UTF-8?B?MDY2MTM5ODEuanBn?=\"").unwrap();
    assert_eq!(parsed.get_key(), "Content-Type");
    assert_eq!(parsed.get_key_ref(), "Content-Type");
    assert_eq!(parsed.get_key_raw(), "Content-Type".as_bytes());
    assert_eq!(parsed.get_value(), "image/jpeg; name=\"06613981.jpg\"");
    assert_eq!(
        parsed.get_value_raw(),
        "image/jpeg; name=\"=?UTF-8?B?MDY2MTM5ODEuanBn?=\"".as_bytes()
    );

    let (parsed, _) = parse_header(
        b"From: =?UTF-8?Q?\"Motorola_Owners=E2=80=99_Forums\"_?=<forums@motorola.com>",
    )
    .unwrap();
    assert_eq!(parsed.get_key(), "From");
    assert_eq!(parsed.get_key_ref(), "From");
    assert_eq!(parsed.get_key_raw(), "From".as_bytes());
    assert_eq!(
        parsed.get_value(),
        "\"Motorola Owners\u{2019} Forums\" <forums@motorola.com>"
    );
}

#[test]
fn encoded_words_and_spaces() {
    let (parsed, _) = parse_header(b"K: an =?utf-8?q?encoded?=\n word").unwrap();
    assert_eq!(parsed.get_value(), "an encoded word");
    assert_eq!(
        parsed.get_value_raw(),
        "an =?utf-8?q?encoded?=\n word".as_bytes()
    );

    let (parsed, _) = parse_header(b"K: =?utf-8?q?glue?= =?utf-8?q?these?= \n words").unwrap();
    assert_eq!(parsed.get_value(), "gluethese  words");
    assert_eq!(
        parsed.get_value_raw(),
        "=?utf-8?q?glue?= =?utf-8?q?these?= \n words".as_bytes()
    );

    let (parsed, _) = parse_header(b"K: =?utf-8?q?glue?= \n =?utf-8?q?again?=").unwrap();
    assert_eq!(parsed.get_value(), "glueagain");
    assert_eq!(
        parsed.get_value_raw(),
        "=?utf-8?q?glue?= \n =?utf-8?q?again?=".as_bytes()
    );
}

#[test]
fn parse_multiple_headers() {
    let (parsed, _) = parse_headers(b"Key: Value\nTwo: Second").unwrap();
    assert_eq!(parsed.len(), 2);
    assert_eq!(parsed[0].key, b"Key");
    assert_eq!(parsed[0].value, b"Value");
    assert_eq!(parsed[1].key, b"Two");
    assert_eq!(parsed[1].value, b"Second");

    let (parsed, _) = parse_headers(b"Key: Value\n Overhang\nTwo: Second\nThree: Third").unwrap();
    assert_eq!(parsed.len(), 3);
    assert_eq!(parsed[0].key, b"Key");
    assert_eq!(parsed[0].value, b"Value\n Overhang");
    assert_eq!(parsed[1].key, b"Two");
    assert_eq!(parsed[1].value, b"Second");
    assert_eq!(parsed[2].key, b"Three");
    assert_eq!(parsed[2].value, b"Third");

    let (parsed, _) = parse_headers(b"Key: Value\nTwo: Second\n\nBody").unwrap();
    assert_eq!(parsed.len(), 2);
    assert_eq!(parsed[0].key, b"Key");
    assert_eq!(parsed[0].value, b"Value");
    assert_eq!(parsed[1].key, b"Two");
    assert_eq!(parsed[1].value, b"Second");

    let (parsed, _) = parse_headers(
        concat!(
            "Return-Path: <kats@foobar.staktrace.com>\n",
            "X-Original-To: kats@baz.staktrace.com\n",
            "Delivered-To: kats@baz.staktrace.com\n",
            "Received: from foobar.staktrace.com (localhost [127.0.0.1])\n",
            "    by foobar.staktrace.com (Postfix) with ESMTP id \
                 139F711C1C34\n",
            "    for <kats@baz.staktrace.com>; Fri, 27 May 2016 02:34:26 \
                 -0400 (EDT)\n",
            "Date: Fri, 27 May 2016 02:34:25 -0400\n",
            "To: kats@baz.staktrace.com\n",
            "From: kats@foobar.staktrace.com\n",
            "Subject: test Fri, 27 May 2016 02:34:25 -0400\n",
            "X-Mailer: swaks v20130209.0 jetmore.org/john/code/swaks/\n",
            "Message-Id: \
                 <20160527063426.139F711C1C34@foobar.staktrace.com>\n",
            "\n",
            "This is a test mailing\n"
        )
        .as_bytes(),
    )
    .unwrap();
    assert_eq!(parsed.len(), 10);
    assert_eq!(parsed[0].key, b"Return-Path");
    assert_eq!(parsed[9].key, b"Message-Id");

    let (parsed, _) =
        parse_headers(b"Key: Value\nAnotherKey: AnotherValue\nKey: Value2\nKey: Value3\n").unwrap();
    assert_eq!(parsed.len(), 4);
    assert_eq!(parsed.get_first_value("Key"), Some("Value".to_string()));
    assert_eq!(
        parsed.get_all_values("Key"),
        vec!["Value", "Value2", "Value3"]
    );
    assert_eq!(
        parsed.get_first_value("AnotherKey"),
        Some("AnotherValue".to_string())
    );
    assert_eq!(parsed.get_all_values("AnotherKey"), vec!["AnotherValue"]);
    assert_eq!(parsed.get_first_value("NoKey"), None);
    assert_eq!(parsed.get_all_values("NoKey"), Vec::<String>::new());

    let (parsed, _) = parse_headers(b"Key: value\r\nWith: CRLF\r\n\r\nBody").unwrap();
    assert_eq!(parsed.len(), 2);
    assert_eq!(parsed.get_first_value("Key"), Some("value".to_string()));
    assert_eq!(parsed.get_first_value("With"), Some("CRLF".to_string()));

    let (parsed, _) = parse_headers(b"Bad\nKey\n").unwrap();
    assert_eq!(parsed.len(), 2);
    assert_eq!(parsed.get_first_value("Bad"), Some("".to_string()));
    assert_eq!(parsed.get_first_value("Key"), Some("".to_string()));

    let (parsed, _) = parse_headers(b"K:V\nBad\nKey").unwrap();
    assert_eq!(parsed.len(), 3);
    assert_eq!(parsed.get_first_value("K"), Some("V".to_string()));
    assert_eq!(parsed.get_first_value("Bad"), Some("".to_string()));
    assert_eq!(parsed.get_first_value("Key"), Some("".to_string()));
}

#[test]
fn test_parse_content_type() {
    let ctype = parse_content_type("text/html; charset=utf-8");
    assert_eq!(ctype.mimetype, "text/html");
    assert_eq!(ctype.charset, "utf-8");
    assert_eq!(ctype.params.get("boundary"), None);

    let ctype = parse_content_type(" foo/bar; x=y; charset=\"fake\" ; x2=y2");
    assert_eq!(ctype.mimetype, "foo/bar");
    assert_eq!(ctype.charset, "fake");
    assert_eq!(ctype.params.get("boundary"), None);

    let ctype = parse_content_type(" multipart/bar; boundary=foo ");
    assert_eq!(ctype.mimetype, "multipart/bar");
    assert_eq!(ctype.charset, "us-ascii");
    assert_eq!(ctype.params.get("boundary").unwrap(), "foo");
}

#[test]
fn test_parse_content_disposition() {
    let dis = parse_content_disposition("inline");
    assert_eq!(dis.disposition, DispositionType::Inline);
    assert_eq!(dis.params.get("name"), None);
    assert_eq!(dis.params.get("filename"), None);

    let dis = parse_content_disposition(
        " attachment; x=y; charset=\"fake\" ; x2=y2; name=\"King Joffrey.death\"",
    );
    assert_eq!(dis.disposition, DispositionType::Attachment);
    assert_eq!(
        dis.params.get("name"),
        Some(&"King Joffrey.death".to_string())
    );
    assert_eq!(dis.params.get("filename"), None);

    let dis = parse_content_disposition(" form-data");
    assert_eq!(dis.disposition, DispositionType::FormData);
    assert_eq!(dis.params.get("name"), None);
    assert_eq!(dis.params.get("filename"), None);
}

#[test]
fn test_parse_mail() {
    let mail = parse_mail(b"Key: value\r\n\r\nSome body stuffs").unwrap();
    assert_eq!(mail.header_bytes, b"Key: value\r\n\r\n");
    assert_eq!(mail.headers.len(), 1);
    assert_eq!(mail.headers[0].get_key(), "Key");
    assert_eq!(mail.headers[0].get_key_ref(), "Key");
    assert_eq!(mail.headers[0].get_value(), "value");
    assert_eq!(mail.ctype.mimetype, "text/plain");
    assert_eq!(mail.ctype.charset, "us-ascii");
    assert_eq!(mail.ctype.params.get("boundary"), None);
    assert_eq!(mail.body_bytes, b"Some body stuffs");
    assert_eq!(mail.get_body_raw().unwrap(), b"Some body stuffs");
    assert_eq!(mail.get_body().unwrap(), "Some body stuffs");
    assert_eq!(mail.subparts.len(), 0);

    let mail = parse_mail(
        concat!(
            "Content-Type: MULTIpart/alternative; bounDAry=myboundary\r\n\r\n",
            "--myboundary\r\n",
            "Content-Type: text/plain\r\n\r\n",
            "This is the plaintext version.\r\n",
            "--myboundary\r\n",
            "Content-Type: text/html;chARset=utf-8\r\n\r\n",
            "This is the <b>HTML</b> version with fake --MYBOUNDARY.\r\n",
            "--myboundary--"
        )
        .as_bytes(),
    )
    .unwrap();
    assert_eq!(mail.headers.len(), 1);
    assert_eq!(mail.headers[0].get_key(), "Content-Type");
    assert_eq!(mail.headers[0].get_key_ref(), "Content-Type");
    assert_eq!(mail.ctype.mimetype, "multipart/alternative");
    assert_eq!(mail.ctype.charset, "us-ascii");
    assert_eq!(mail.ctype.params.get("boundary").unwrap(), "myboundary");
    assert_eq!(mail.subparts.len(), 2);
    assert_eq!(mail.subparts[0].headers.len(), 1);
    assert_eq!(mail.subparts[0].ctype.mimetype, "text/plain");
    assert_eq!(mail.subparts[0].ctype.charset, "us-ascii");
    assert_eq!(mail.subparts[0].ctype.params.get("boundary"), None);
    assert_eq!(mail.subparts[1].ctype.mimetype, "text/html");
    assert_eq!(mail.subparts[1].ctype.charset, "utf-8");
    assert_eq!(mail.subparts[1].ctype.params.get("boundary"), None);

    let mail =
        parse_mail(b"Content-Transfer-Encoding: base64\r\n\r\naGVsbG 8gd\r\n29ybGQ=").unwrap();
    assert_eq!(mail.get_body_raw().unwrap(), b"hello world");
    assert_eq!(mail.get_body().unwrap(), "hello world");

    let mail =
        parse_mail(b"Content-Type: text/plain; charset=x-unknown\r\n\r\nhello world").unwrap();
    assert_eq!(mail.get_body_raw().unwrap(), b"hello world");
    assert_eq!(mail.get_body().unwrap(), "hello world");

    let mail = parse_mail(b"ConTENT-tyPE: text/html\r\n\r\nhello world").unwrap();
    assert_eq!(mail.ctype.mimetype, "text/html");
    assert_eq!(mail.get_body_raw().unwrap(), b"hello world");
    assert_eq!(mail.get_body().unwrap(), "hello world");

    let mail = parse_mail(
        b"Content-Type: text/plain; charset=UTF-7\r\nContent-Transfer-Encoding: quoted-printable\r\n\r\n+JgM-",
    ).unwrap();
    assert_eq!(mail.get_body_raw().unwrap(), b"+JgM-");
    assert_eq!(mail.get_body().unwrap(), "\u{2603}");

    let mail = parse_mail(b"Content-Type: text/plain; charset=UTF-7\r\n\r\n+JgM-").unwrap();
    assert_eq!(mail.get_body_raw().unwrap(), b"+JgM-");
    assert_eq!(mail.get_body().unwrap(), "\u{2603}");
}

#[test]
fn test_missing_terminating_boundary() {
    let mail = parse_mail(
        concat!(
            "Content-Type: multipart/alternative; boundary=myboundary\r\n\r\n",
            "--myboundary\r\n",
            "Content-Type: text/plain\r\n\r\n",
            "part0\r\n",
            "--myboundary\r\n",
            "Content-Type: text/html\r\n\r\n",
            "part1\r\n"
        )
        .as_bytes(),
    )
    .unwrap();
    assert_eq!(mail.subparts[0].get_body().unwrap(), "part0\r\n");
    assert_eq!(mail.subparts[1].get_body().unwrap(), "part1\r\n");
}

#[test]
fn test_missing_body() {
    let parsed =
        parse_mail("Content-Type: multipart/related; boundary=\"----=_\"\n".as_bytes()).unwrap();
    assert_eq!(parsed.headers[0].get_key(), "Content-Type");
    assert_eq!(parsed.get_body_raw().unwrap(), b"");
    assert_eq!(parsed.get_body().unwrap(), "");
}

#[test]
fn test_no_headers_in_subpart() {
    let mail = parse_mail(
        concat!(
            "Content-Type: multipart/report; report-type=delivery-status;\n",
            "\tboundary=\"1404630116.22555.postech.q0.x.x.x\"\n",
            "\n",
            "--1404630116.22555.postech.q0.x.x.x\n",
            "\n",
            "--1404630116.22555.postech.q0.x.x.x--\n"
        )
        .as_bytes(),
    )
    .unwrap();
    assert_eq!(mail.ctype.mimetype, "multipart/report");
    assert_eq!(mail.subparts[0].headers.len(), 0);
    assert_eq!(mail.subparts[0].ctype.mimetype, "text/plain");
    assert_eq!(mail.subparts[0].get_body_raw().unwrap(), b"");
    assert_eq!(mail.subparts[0].get_body().unwrap(), "");
}

#[test]
fn test_empty() {
    let mail = parse_mail("".as_bytes()).unwrap();
    assert_eq!(mail.get_body_raw().unwrap(), b"");
    assert_eq!(mail.get_body().unwrap(), "");
}

#[test]
fn test_dont_panic_for_value_with_new_lines() {
    let parsed = parse_param_content(r#"application/octet-stream; name=""#);
    assert_eq!(parsed.params["name"], "\"");
}

#[test]
fn test_parameter_value_continuations() {
    let parsed = parse_param_content("attachment;\n\tfilename*0=\"X\";\n\tfilename*1=\"Y.pdf\"");
    assert_eq!(parsed.value, "attachment");
    assert_eq!(parsed.params["filename"], "XY.pdf");
    assert!(!parsed.params.contains_key("filename*0"));
    assert!(!parsed.params.contains_key("filename*1"));

    let parsed = parse_param_content(
        "attachment;\n\tfilename=XX.pdf;\n\tfilename*0=\"X\";\n\tfilename*1=\"Y.pdf\"",
    );
    assert_eq!(parsed.value, "attachment");
    assert_eq!(parsed.params["filename"], "XX.pdf");
    assert_eq!(parsed.params["filename*0"], "X");
    assert_eq!(parsed.params["filename*1"], "Y.pdf");

    let parsed = parse_param_content("attachment; filename*1=\"Y.pdf\"");
    assert_eq!(parsed.params["filename*1"], "Y.pdf");
    assert!(!parsed.params.contains_key("filename"));
}

#[test]
fn test_parameter_encodings() {
    let parsed = parse_param_content("attachment;\n\tfilename*0*=us-ascii''%28X%29%20801%20-%20X;\n\tfilename*1*=%20%E2%80%93%20X%20;\n\tfilename*2*=X%20X%2Epdf");
    // Note this is a real-world case from mutt, but it's wrong. The original filename had an en dash \u{2013} but mutt
    // declared us-ascii as the encoding instead of utf-8 for some reason.
    assert_eq!(
        parsed.params["filename"],
        "(X) 801 - X \u{00E2}\u{20AC}\u{201C} X X X.pdf"
    );
    assert!(!parsed.params.contains_key("filename*0*"));
    assert!(!parsed.params.contains_key("filename*0"));
    assert!(!parsed.params.contains_key("filename*1*"));
    assert!(!parsed.params.contains_key("filename*1"));
    assert!(!parsed.params.contains_key("filename*2*"));
    assert!(!parsed.params.contains_key("filename*2"));

    // Here is the corrected version.
    let parsed = parse_param_content("attachment;\n\tfilename*0*=utf-8''%28X%29%20801%20-%20X;\n\tfilename*1*=%20%E2%80%93%20X%20;\n\tfilename*2*=X%20X%2Epdf");
    assert_eq!(parsed.params["filename"], "(X) 801 - X \u{2013} X X X.pdf");
    assert!(!parsed.params.contains_key("filename*0*"));
    assert!(!parsed.params.contains_key("filename*0"));
    assert!(!parsed.params.contains_key("filename*1*"));
    assert!(!parsed.params.contains_key("filename*1"));
    assert!(!parsed.params.contains_key("filename*2*"));
    assert!(!parsed.params.contains_key("filename*2"));
    let parsed = parse_param_content("attachment; filename*=utf-8'en'%e2%80%A1.bin");
    assert_eq!(parsed.params["filename"], "\u{2021}.bin");
    assert!(!parsed.params.contains_key("filename*"));

    let parsed = parse_param_content("attachment; filename*='foo'%e2%80%A1.bin");
    assert_eq!(parsed.params["filename*"], "'foo'%e2%80%A1.bin");
    assert!(!parsed.params.contains_key("filename"));

    let parsed = parse_param_content("attachment; filename*=nonexistent'foo'%e2%80%a1.bin");
    assert_eq!(parsed.params["filename*"], "nonexistent'foo'%e2%80%a1.bin");
    assert!(!parsed.params.contains_key("filename"));

    let parsed = parse_param_content(
        "attachment; filename*0*=utf-8'en'%e2%80%a1; filename*1*=%e2%80%A1.bin",
    );
    assert_eq!(parsed.params["filename"], "\u{2021}\u{2021}.bin");
    assert!(!parsed.params.contains_key("filename*0*"));
    assert!(!parsed.params.contains_key("filename*0"));
    assert!(!parsed.params.contains_key("filename*1*"));
    assert!(!parsed.params.contains_key("filename*1"));

    let parsed =
        parse_param_content("attachment; filename*0*=utf-8'en'%e2%80%a1; filename*1=%20.bin");
    assert_eq!(parsed.params["filename"], "\u{2021}%20.bin");
    assert!(!parsed.params.contains_key("filename*0*"));
    assert!(!parsed.params.contains_key("filename*0"));
    assert!(!parsed.params.contains_key("filename*1*"));
    assert!(!parsed.params.contains_key("filename*1"));

    let parsed =
        parse_param_content("attachment; filename*0*=utf-8'en'%e2%80%a1; filename*2*=%20.bin");
    assert_eq!(parsed.params["filename"], "\u{2021}");
    assert_eq!(parsed.params["filename*2"], " .bin");
    assert!(!parsed.params.contains_key("filename*0*"));
    assert!(!parsed.params.contains_key("filename*0"));
    assert!(!parsed.params.contains_key("filename*2*"));

    let parsed =
        parse_param_content("attachment; filename*0*=utf-8'en'%e2%80%a1; filename*0=foo.bin");
    assert_eq!(parsed.params["filename"], "foo.bin");
    assert_eq!(parsed.params["filename*0*"], "utf-8'en'%e2%80%a1");
    assert!(!parsed.params.contains_key("filename*0"));
}

#[test]
fn test_default_content_encoding() {
    let mail = parse_mail(b"Content-Type: text/plain; charset=UTF-7\r\n\r\n+JgM-").unwrap();
    let body = mail.get_body_encoded();
    match body {
        Body::SevenBit(body) => {
            assert_eq!(body.get_raw(), b"+JgM-");
            assert_eq!(body.get_as_string().unwrap(), "\u{2603}");
        }
        _ => unreachable!(),
    };
}

#[test]
fn test_7bit_content_encoding() {
    let mail = parse_mail(
        b"Content-Type: text/plain; charset=UTF-7\r\nContent-Transfer-Encoding: 7bit\r\n\r\n+JgM-",
    )
    .unwrap();
    let body = mail.get_body_encoded();
    match body {
        Body::SevenBit(body) => {
            assert_eq!(body.get_raw(), b"+JgM-");
            assert_eq!(body.get_as_string().unwrap(), "\u{2603}");
        }
        _ => unreachable!(),
    };
}

#[test]
fn test_8bit_content_encoding() {
    let mail = parse_mail(
        b"Content-Type: text/plain; charset=UTF-7\r\nContent-Transfer-Encoding: 8bit\r\n\r\n+JgM-",
    )
    .unwrap();
    let body = mail.get_body_encoded();
    match body {
        Body::EightBit(body) => {
            assert_eq!(body.get_raw(), b"+JgM-");
            assert_eq!(body.get_as_string().unwrap(), "\u{2603}");
        }
        _ => unreachable!(),
    };
}

#[test]
fn test_quoted_printable_content_encoding() {
    let mail = parse_mail(
        b"Content-Type: text/plain; charset=UTF-7\r\nContent-Transfer-Encoding: quoted-printable\r\n\r\n+JgM-",
    ).unwrap();
    match mail.get_body_encoded() {
        Body::QuotedPrintable(body) => {
            assert_eq!(body.get_raw(), b"+JgM-");
            assert_eq!(body.get_decoded().unwrap(), b"+JgM-");
            assert_eq!(body.get_decoded_as_string().unwrap(), "\u{2603}");
        }
        _ => unreachable!(),
    };
}

#[test]
fn test_base64_content_encoding() {
    let mail =
        parse_mail(b"Content-Transfer-Encoding: base64\r\n\r\naGVsbG 8gd\r\n29ybGQ=").unwrap();
    match mail.get_body_encoded() {
        Body::Base64(body) => {
            assert_eq!(body.get_raw(), b"aGVsbG 8gd\r\n29ybGQ=");
            assert_eq!(body.get_decoded().unwrap(), b"hello world");
            assert_eq!(body.get_decoded_as_string().unwrap(), "hello world");
        }
        _ => unreachable!(),
    };
}

#[test]
fn test_base64_content_encoding_multiple_strings() {
    let mail =
        parse_mail(b"Content-Transfer-Encoding: base64\r\n\r\naGVsbG 8gd\r\n29ybGQ=\r\nZm9vCg==")
            .unwrap();
    match mail.get_body_encoded() {
        Body::Base64(body) => {
            assert_eq!(body.get_raw(), b"aGVsbG 8gd\r\n29ybGQ=\r\nZm9vCg==");
            assert_eq!(body.get_decoded().unwrap(), b"hello worldfoo\n");
            assert_eq!(body.get_decoded_as_string().unwrap(), "hello worldfoo\n");
        }
        _ => unreachable!(),
    };
}

#[test]
fn test_binary_content_encoding() {
    let mail = parse_mail(b"Content-Transfer-Encoding: binary\r\n\r\n######").unwrap();
    let body = mail.get_body_encoded();
    match body {
        Body::Binary(body) => {
            assert_eq!(body.get_raw(), b"######");
        }
        _ => unreachable!(),
    };
}

#[test]
fn test_body_content_encoding_with_multipart() {
    let mail_filepath = "./tests/files/test_email_01.txt";
    let mail = std::fs::read(mail_filepath)
        .unwrap_or_else(|_| panic!("Unable to open the file [{}]", mail_filepath));
    let mail = parse_mail(&mail).unwrap();

    let subpart_0 = mail.subparts.first().unwrap();
    match subpart_0.get_body_encoded() {
        Body::SevenBit(body) => {
            assert_eq!(
                body.get_as_string().unwrap().trim(),
                "<html>Test with attachments</html>"
            );
        }
        _ => unreachable!(),
    };

    let subpart_1 = mail.subparts.get(1).unwrap();
    match subpart_1.get_body_encoded() {
        Body::Base64(body) => {
            let pdf_filepath = "./tests/files/test_email_01_sample.pdf";
            let original_pdf = std::fs::read(pdf_filepath)
                .unwrap_or_else(|_| panic!("Unable to open the file [{}]", pdf_filepath));
            assert_eq!(body.get_decoded().unwrap(), original_pdf);
        }
        _ => unreachable!(),
    };

    let subpart_2 = mail.subparts.get(2).unwrap();
    match subpart_2.get_body_encoded() {
        Body::Base64(body) => {
            assert_eq!(
                body.get_decoded_as_string().unwrap(),
                "txt file context for email collector\n1234567890987654321\n"
            );
        }
        _ => unreachable!(),
    };
}

#[test]
fn test_fuzzer_testcase() {
    const INPUT: &str = "U3ViamVjdDplcy1UeXBlOiBtdW50ZW50LVV5cGU6IW11bAAAAAAAAAAAamVjdDplcy1UeXBlOiBtdW50ZW50LVV5cGU6IG11bAAAAAAAAAAAAAAAAABTTUFZdWJqZf86OiP/dCBTdWJqZWN0Ol8KRGF0ZTog/////////////////////wAAAAAAAAAAAHQgYnJmAHQgYnJmZXItRW5jeXBlOnY9NmU3OjA2OgAAAAAAAAAAAAAAADEAAAAAAP/8mAAAAAAAAAAA+f///wAAAAAAAP8AAAAAAAAAAAAAAAAAAAAAAAAAPT0/PzEAAAEAAA==";

    if let Ok(parsed) = parse_mail(&data_encoding::BASE64.decode(INPUT.as_bytes()).unwrap()) {
        if let Some(date) = parsed.headers.get_first_value("Date") {
            let _ = dateparse(&date);
        }
    }
}

#[test]
fn test_fuzzer_testcase_2() {
    const INPUT: &str = "U3ViamVjdDogVGhpcyBpcyBhIHRlc3QgZW1haWwKQ29udGVudC1UeXBlOiBtdWx0aXBhcnQvYWx0ZXJuYXRpdmU7IGJvdW5kYXJ5PczMzMzMzMzMzMzMzMzMzMzMzMzMzMzMzMzMzMzMzMzMzMzMzMzMzMzMzMzMzMzMzMzMzMzMzMzMzMzMzMzMzMzMzMzMzMzMzMzMzMzMzMzMzMzMzMzMzMzMzMzMzMzMzMzMzMzMzMzMzMzMzMzMzMzMZm9vYmFyCkRhdGU6IFN1biwgMDIgT2MKCi1TdWJqZWMtZm9vYmFydDo=";
    if let Ok(parsed) = parse_mail(&data_encoding::BASE64.decode(INPUT.as_bytes()).unwrap()) {
        if let Some(date) = parsed.headers.get_first_value("Date") {
            let _ = dateparse(&date);
        }
    }
}

#[test]
fn test_header_split() {
    let mail = parse_mail(
        b"Content-Type: text/plain;\r\ncharset=\"utf-8\"\r\nContent-Transfer-Encoding: 8bit\r\n\r\n",
    ).unwrap();
    assert_eq!(mail.ctype.mimetype, "text/plain");
    assert_eq!(mail.ctype.charset, "us-ascii");
}

#[test]
fn test_percent_decoder() {
    assert_eq!(percent_decode("hi %0d%0A%%2A%zz%"), b"hi \r\n%*%zz%");
}

#[test]
fn test_default_content_type_in_multipart_digest() {
    // Per https://datatracker.ietf.org/doc/html/rfc2046#section-5.1.5
    let mail = parse_mail(
        concat!(
            "Content-Type: multipart/digest; boundary=myboundary\r\n\r\n",
            "--myboundary\r\n\r\n",
            "blah blah blah\r\n\r\n",
            "--myboundary--\r\n"
        )
        .as_bytes(),
    )
    .unwrap();
    assert_eq!(mail.headers.len(), 1);
    assert_eq!(mail.ctype.mimetype, "multipart/digest");
    assert_eq!(mail.subparts[0].headers.len(), 0);
    assert_eq!(mail.subparts[0].ctype.mimetype, "message/rfc822");

    let mail = parse_mail(
        concat!(
            "Content-Type: multipart/whatever; boundary=myboundary\n",
            "\n",
            "--myboundary\n",
            "\n",
            "blah blah blah\n",
            "--myboundary\n",
            "Content-Type: multipart/digest; boundary=nestedboundary\n",
            "\n",
            "--nestedboundary\n",
            "\n",
            "nested default part\n",
            "--nestedboundary\n",
            "Content-Type: text/html\n",
            "\n",
            "nested html part\n",
            "--nestedboundary\n",
            "Content-Type: multipart/insidedigest; boundary=insideboundary\n",
            "\n",
            "--insideboundary\n",
            "\n",
            "inside part\n",
            "--insideboundary--\n",
            "--nestedboundary--\n",
            "--myboundary--\n"
        )
        .as_bytes(),
    )
    .unwrap();
    let mut parts = mail.parts();
    let mut part = parts.next().unwrap(); // mail

    assert_eq!(part.headers.len(), 1);
    assert_eq!(part.ctype.mimetype, "multipart/whatever");

    part = parts.next().unwrap(); // mail.subparts[0]
    assert_eq!(part.headers.len(), 0);
    assert_eq!(part.ctype.mimetype, "text/plain");
    assert_eq!(part.get_body_raw().unwrap(), b"blah blah blah\n");

    part = parts.next().unwrap(); // mail.subparts[1]
    assert_eq!(part.ctype.mimetype, "multipart/digest");

    part = parts.next().unwrap(); // mail.subparts[1].subparts[0]
    assert_eq!(part.headers.len(), 0);
    assert_eq!(part.ctype.mimetype, "message/rfc822");
    assert_eq!(part.get_body_raw().unwrap(), b"nested default part\n");

    part = parts.next().unwrap(); // mail.subparts[1].subparts[1]
    assert_eq!(part.headers.len(), 1);
    assert_eq!(part.ctype.mimetype, "text/html");
    assert_eq!(part.get_body_raw().unwrap(), b"nested html part\n");

    part = parts.next().unwrap(); // mail.subparts[1].subparts[2]
    assert_eq!(part.headers.len(), 1);
    assert_eq!(part.ctype.mimetype, "multipart/insidedigest");

    part = parts.next().unwrap(); // mail.subparts[1].subparts[2].subparts[0]
    assert_eq!(part.headers.len(), 0);
    assert_eq!(part.ctype.mimetype, "text/plain");
    assert_eq!(part.get_body_raw().unwrap(), b"inside part\n");

    assert!(parts.next().is_none());
}

#[test]
fn boundary_is_suffix_of_another_boundary() {
    // From https://github.com/staktrace/mailparse/issues/100
    let mail = parse_mail(
        concat!(
            "Content-Type: multipart/mixed; boundary=\"section_boundary\"\n",
            "\n",
            "--section_boundary\n",
            "Content-Type: multipart/alternative; boundary=\"--section_boundary\"\n",
            "\n",
            "----section_boundary\n",
            "Content-Type: text/html;\n",
            "\n",
            "<em>Good evening!</em>\n",
            "----section_boundary\n",
            "Content-Type: text/plain;\n",
            "\n",
            "Good evening!\n",
            "----section_boundary\n",
            "--section_boundary\n"
        )
        .as_bytes(),
    )
    .unwrap();

    let mut parts = mail.parts();
    let mut part = parts.next().unwrap(); // mail

    assert_eq!(part.headers.len(), 1);
    assert_eq!(part.ctype.mimetype, "multipart/mixed");
    assert_eq!(part.subparts.len(), 1);

    part = parts.next().unwrap(); // mail.subparts[0]
    assert_eq!(part.headers.len(), 1);
    assert_eq!(part.ctype.mimetype, "multipart/alternative");
    assert_eq!(part.subparts.len(), 2);

    part = parts.next().unwrap(); // mail.subparts[0].subparts[0]
    assert_eq!(part.headers.len(), 1);
    assert_eq!(part.ctype.mimetype, "text/html");
    assert_eq!(part.get_body_raw().unwrap(), b"<em>Good evening!</em>\n");
    assert_eq!(part.subparts.len(), 0);

    part = parts.next().unwrap(); // mail.subparts[0].subparts[1]
    assert_eq!(part.headers.len(), 1);
    assert_eq!(part.ctype.mimetype, "text/plain");
    assert_eq!(part.get_body_raw().unwrap(), b"Good evening!\n");
    assert_eq!(part.subparts.len(), 0);

    assert!(parts.next().is_none());
}

#[test]
fn test_parts_iterator() {
    let mail = parse_mail(
        concat!(
            "Content-Type: multipart/mixed; boundary=\"top_boundary\"\n",
            "\n",
            "--top_boundary\n",
            "Content-Type: multipart/alternative; boundary=\"internal_boundary\"\n",
            "\n",
            "--internal_boundary\n",
            "Content-Type: text/html;\n",
            "\n",
            "<em>Good evening!</em>\n",
            "--internal_boundary\n",
            "Content-Type: text/plain;\n",
            "\n",
            "Good evening!\n",
            "--internal_boundary\n",
            "--top_boundary\n",
            "Content-Type: text/unknown;\n",
            "\n",
            "You read this?\n",
            "--top_boundary\n"
        )
        .as_bytes(),
    )
    .unwrap();

    let mut parts = mail.parts();
    assert_eq!(parts.next().unwrap().ctype.mimetype, "multipart/mixed");
    assert_eq!(
        parts.next().unwrap().ctype.mimetype,
        "multipart/alternative"
    );
    assert_eq!(parts.next().unwrap().ctype.mimetype, "text/html");
    assert_eq!(parts.next().unwrap().ctype.mimetype, "text/plain");
    assert_eq!(parts.next().unwrap().ctype.mimetype, "text/unknown");
    assert!(parts.next().is_none());

    let mail = parse_mail(concat!("Content-Type: text/plain\n").as_bytes()).unwrap();

    let mut parts = mail.parts();
    assert_eq!(parts.next().unwrap().ctype.mimetype, "text/plain");
    assert!(parts.next().is_none());
}
