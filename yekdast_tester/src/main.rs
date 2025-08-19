use std::collections::HashMap;
use yekdast::{normalize_text, NormalizeOptions, DigitPolicy};

fn main() {
    println!("--- اجرای برنامه تست کتابخانه Yekdast ---");
    
    // متنی با انواع مشکلات: حروف عربی، اعداد لاتین، محاوره‌ای، بدون نیم‌فاصله و...
    let messy_text = "من توی خونه ي خودم با شماره تلفن 09123456789 كار ميكنم و به كتاب خانه علاقه مند هستم.";

    println!("\n[ورودی]");
    println!("متن اصلی: {}", messy_text);
    
    // --- ساخت تنظیمات سفارشی ---
    
    // دیکشنری برای تبدیل کلمات محاوره‌ای به رسمی
    let mut slang_map = HashMap::new();
    slang_map.insert("توی".to_string(), "در".to_string());
    slang_map.insert("خونه".to_string(), "خانه".to_string());
    
    // لیست کلمات مرکبی که باید نیم‌فاصله بگیرند
    let zwnj_words = vec![
        "کار میکنم".to_string(),
        "کتاب خانه".to_string(),
        "علاقه مند".to_string(),
    ];
    
    // ساختن نمونه تنظیمات
    let options = NormalizeOptions {
        digits: DigitPolicy::Fa, // اعداد را به فارسی تبدیل کن
        slang_map,
        zwnj_compound_words: zwnj_words,
        ..Default::default()
    };
    
    // فراخوانی تابع اصلی کتابخانه
    let clean_text = normalize_text(messy_text, &options);
    
    println!("\n[خروجی]");
    println!("متن اصلاح شده: {}", clean_text);
    println!("\n--- تست پایان یافت ---");
}
