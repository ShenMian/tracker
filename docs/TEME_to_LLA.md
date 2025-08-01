# TEME to LLA

## 流程

1. UTC 转 TAI

    $$
    \begin{aligned}
        \text{UTC} - \text{TAI} = -37 \text{s} \\
        \text{TAI} = \text{UTC} + 37 \text{s} \\
    \end{aligned}
    $$

    这一步消除了闰秒带来的误差影响.  
    数据来源于 [Bulletin C 69] (截至 2025 年).

2. TAI 转 TT

    $\text{TT} = \text{TAI} + 32.184$

    这一步转换为天体力学 (如轨道传播) 常用的时间尺度.

```mermaid
flowchart LR
    A[TLE/OMM] -->|解析| ELEM[轨道根数]
    ELEM --> Epoch
    Epoch --> UTC["JD (UTC)"]

    subgraph "时间尺度转换"
        UTC -->|+闰秒| TAI["JD (TAI)"]
        TAI -->|+32.184 秒| TT["JD (TT)"]
    end

    TT --> GMST
    GMST -->|构造| R[旋转矩阵]

    ELEM --> SGP4
    SGP4 -->|轨道传播| TEME

    subgraph "坐标转换"
        TEME -->|相乘| R
        R --> ECEF
        ECEF --> LLA
    end
```

[Bulletin C 69]: https://datacenter.iers.org/data/html/bulletinc-069.html

## 术语表

| 缩写 | 中文               | 英文                               |
|------|--------------------|------------------------------------|
| TEME | 真赤道平春分坐标系 | True Equator Mean Equinox          |
| ECEF | 地心地固坐标系     | Earth Centered Earth Fixed         |
| LLA  | 大地坐标/经纬高    | Latitude, Longitude, Altitude      |
| JD   | 儒略日             | Julian Day                         |
| UTC  | 协调世界时         | Coordinated Universal Time         |
| TAI  | 国际原子时         | International Atomic Time          |
| TT   | 地球时             | Terrestrial Time                   |
| UT1  | 世界时             | Universal Time                     |
| GMST | 格林尼治平恒星时   | Greenwich Mean Sidereal Time       |
| TLE  | 两行轨道根数       | Two-Line Element Set               |
| OMM  | 轨道平均根数信息   | Orbit Mean-Elements Message        |
| SGP4 |                    | Simplified General Perturbations 4 |

## 参见

- <https://aa.usno.navy.mil/faq/GAST>.
