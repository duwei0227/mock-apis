use chrono::Local;
use rand::Rng;
use uuid::Uuid;

/// 现代汉语常用字表 — ~1000 most-used simplified Chinese characters.
/// Covers the vast majority of everyday Chinese text.
const COMMON_CHINESE_CHARS: &str = concat!(
    // 高频基础字 (highest frequency)
    "的一是在不了有和人这中大为上个国我以要他时来用们生到作地于出就分对成会可主发年动同工也能下",
    "过子说产种面而方后多定行学法所民得经十三之进着等部度家电力里如水化高自二理起小物现实加量都",
    "两体制机当使点从业本去把性好应开它合还因由其些然前外天政四日那社义事平形相全表间样与关各重",
    "新线内数正心力无已当收感次系明信看提问先做件接报带结没每公原你月好意最行单员走何字特总目通",
    // 常用字第二级
    "保始建式程规完论代活太变世展整据处运研管教较图指将认决则情化向基转单步所调设计组长手记市美",
    "特区起命色比期解知名该立示快别百支确级质改强调力达连路真般属立花科技解史标格革命属科技路真",
    "般花史格值示权解放院委委书记党军队长首称号级科处级段话求院校场院医馆区站台港桥路街道村镇县",
    "市省区级党委政府军队机关团体单位公司企业学校医院工厂农场银行商店餐厅旅馆宾馆机场港口码头",
    // 人、社会、关系类
    "父母兄弟姐妹夫妻儿女朋友同志领导群众阶级民族人民公民百姓官员战士教师医生工人农民学生商人",
    // 时间、空间
    "今昨明早晚午夜朝暮春夏秋冬前后左右东西南北上下内外远近高低长短快慢早晚新旧大小多少",
    // 动作、状态
    "走跑跳飞游泳唱跳笑哭爱恨喜怒哀乐看听说读写画想知道理解记忘学习工作休息睡醒吃喝穿脱拿放",
    "开关进出上下来去回送接推拉打击切断折弯拉推摔扔抓握举放送接拖拉揉搓按摩捏挤压折断切割",
    // 描述、形容
    "美丑善恶真假虚实强弱软硬冷热明暗轻重厚薄宽窄深浅清浊纯杂整乱干净脏白黑红绿蓝黄紫橙粉灰棕",
    // 数量、程度
    "零一二三四五六七八九十百千万亿些许多少很太极非常特别尤其格外相当比较稍微略微大约几乎",
    // 连接、逻辑
    "和或但而且并且虽然但是因为所以如果那么只有才能既然就算即使哪怕除非否则不管无论反正总之",
    // 常见名词
    "水火土木金风雨雪云雷电山河海湖江河树草花果根茎叶种子动物植物细菌病毒空气土地阳光温度",
    "家庭社会国家民族世界历史文化教育科学技术经济政治法律军事艺术体育卫生环境资源能源信息网络"
);

/// Render all `{{fn}}` / `{{fn:arg}}` / `{{fn:arg1:arg2}}` placeholders in `body`.
pub fn render(body: &str) -> String {
    let mut result = String::with_capacity(body.len() + 32);
    let bytes = body.as_bytes();
    let len = bytes.len();
    let mut i = 0;

    while i < len {
        if i + 1 < len && bytes[i] == b'{' && bytes[i + 1] == b'{' {
            // scan for closing '}}'
            let start = i + 2;
            let mut j = start;
            while j + 1 < len && !(bytes[j] == b'}' && bytes[j + 1] == b'}') {
                j += 1;
            }
            if j + 1 < len {
                let expr = &body[start..j];
                result.push_str(&eval(expr));
                i = j + 2;
            } else {
                // no closing found — emit literally
                result.push('{');
                result.push('{');
                i += 2;
            }
        } else {
            result.push(body[i..].chars().next().unwrap_or(' '));
            i += body[i..].chars().next().map(|c| c.len_utf8()).unwrap_or(1);
        }
    }
    result
}

fn eval(expr: &str) -> String {
    let parts: Vec<&str> = expr.split(':').collect();
    let func = parts[0].trim();
    match func {
        "date" => {
            let fmt = parts.get(1).map(|s| java_to_strftime(s)).unwrap_or_else(|| "%Y%m%d".into());
            Local::now().format(&fmt).to_string()
        }
        "time" => {
            let fmt = parts.get(1).map(|s| java_to_strftime(s)).unwrap_or_else(|| "%H%M%S".into());
            Local::now().format(&fmt).to_string()
        }
        "datetime" => {
            let fmt = parts.get(1).map(|s| java_to_strftime(s)).unwrap_or_else(|| "%Y%m%d%H%M%S".into());
            Local::now().format(&fmt).to_string()
        }
        "randomInt" => {
            let min: i64 = parts.get(1).and_then(|s| s.parse().ok()).unwrap_or(0);
            let max: i64 = parts.get(2).and_then(|s| s.parse().ok()).unwrap_or(100);
            rand::thread_rng().gen_range(min..=max).to_string()
        }
        "randomFloat" => {
            let min: f64 = parts.get(1).and_then(|s| s.parse().ok()).unwrap_or(0.0);
            let max: f64 = parts.get(2).and_then(|s| s.parse().ok()).unwrap_or(1.0);
            let dec: usize = parts.get(3).and_then(|s| s.parse().ok()).unwrap_or(2);
            format!("{:.prec$}", rand::thread_rng().gen_range(min..max), prec = dec)
        }
        "randomString" => {
            let n: usize = parts.get(1).and_then(|s| s.parse().ok()).unwrap_or(10);
            let charset = b"abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789";
            let mut rng = rand::thread_rng();
            (0..n).map(|_| charset[rng.gen_range(0..charset.len())] as char).collect()
        }
        "randomChinese" => {
            let n: usize = parts.get(1).and_then(|s| s.parse().ok()).unwrap_or(15);
            // Deduplicate at call time (cheap for this pool size).
            let mut seen = std::collections::HashSet::new();
            let chars: Vec<char> = COMMON_CHINESE_CHARS.chars().filter(|c| seen.insert(*c)).collect();
            let mut rng = rand::thread_rng();
            (0..n).map(|_| chars[rng.gen_range(0..chars.len())]).collect()
        }
        "uuid" => Uuid::new_v4().to_string(),
        _ => format!("{{{{{}}}}}", expr), // unknown → keep as-is
    }
}

/// Convert Java-style date format to strftime format.
fn java_to_strftime(fmt: &str) -> String {
    fmt.replace("yyyy", "%Y")
        .replace("MM", "%m")
        .replace("dd", "%d")
        .replace("HH", "%H")
        .replace("mm", "%M")
        .replace("SS", "%S")
        .replace("ss", "%S")
}

/// All built-in functions for display in help panels.
pub struct FnDoc {
    pub name: &'static str,
    pub syntax: &'static str,
    pub default_args: &'static str,
    pub description: &'static str,
    pub example_output: &'static str,
}

pub const FUNCTIONS: &[FnDoc] = &[
    FnDoc {
        name: "date",
        syntax: "{{date}} or {{date:format}}",
        default_args: "yyyyMMdd",
        description: "Current local date",
        example_output: "20260503",
    },
    FnDoc {
        name: "time",
        syntax: "{{time}} or {{time:format}}",
        default_args: "HHmmss",
        description: "Current local time",
        example_output: "143025",
    },
    FnDoc {
        name: "datetime",
        syntax: "{{datetime}} or {{datetime:format}}",
        default_args: "yyyyMMddHHmmss",
        description: "Current local date + time",
        example_output: "20260503143025",
    },
    FnDoc {
        name: "randomInt",
        syntax: "{{randomInt}} or {{randomInt:min:max}}",
        default_args: "0 to 100",
        description: "Random integer in range",
        example_output: "42",
    },
    FnDoc {
        name: "randomFloat",
        syntax: "{{randomFloat}} or {{randomFloat:min:max:decimals}}",
        default_args: "0.0 to 1.0, 2 decimals",
        description: "Random float in range",
        example_output: "0.73",
    },
    FnDoc {
        name: "randomString",
        syntax: "{{randomString}} or {{randomString:length}}",
        default_args: "10 characters",
        description: "Random alphanumeric string",
        example_output: "aB3kFz9Qmw",
    },
    FnDoc {
        name: "randomChinese",
        syntax: "{{randomChinese}} or {{randomChinese:length}}",
        default_args: "15 characters",
        description: "Random simplified Chinese characters",
        example_output: "的一是在人有我",
    },
    FnDoc {
        name: "uuid",
        syntax: "{{uuid}}",
        default_args: "—",
        description: "Random UUID v4",
        example_output: "550e8400-e29b-41d4-a716-446655440000",
    },
];
