use maxminddb::geoip2::City;
use maxminddb::Reader;
use std::net::IpAddr;

pub fn position() -> Result<(), Box<dyn std::error::Error>> {
    let reader = Reader::open_readfile("GeoLite2-City.mmdb")?;
    let ip: IpAddr = "8.8.8.8".parse()?;
    
    // 修复?操作符错误
    let city: City = reader.lookup(ip)?
        .ok_or("IP lookup failed: no city data found")?;
    
    println!(
        "国家: {}, 城市: {}, 经纬度: ({:?}, {:?})",
        // 优化Option处理
        city.country.as_ref()
            .and_then(|c| c.names.as_ref())
            .and_then(|n| n.get("zh-CN").map(|s| s.to_string()))
            .unwrap_or_else(|| "N/A".into()),
        
        city.city.as_ref()
            .and_then(|c| c.names.as_ref())
            .and_then(|n| n.get("zh-CN").map(|s| s.to_string()))
            .unwrap_or_else(|| "N/A".into()),
        
        city.location.as_ref().and_then(|loc| loc.latitude),
        city.location.as_ref().and_then(|loc| loc.longitude)
    );
    Ok(())
}
