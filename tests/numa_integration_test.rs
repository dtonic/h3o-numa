#[cfg(all(feature = "numa", feature = "geo"))]
mod numa_integration_tests {
    use geo::{LineString, Polygon};
    use h3on::{geom::ContainmentMode, Resolution};

    #[test]
    fn test_polygon_to_cells_numa_integration() {
        // 간단한 사각형 폴리곤 생성
        let polygon = Polygon::new(
            LineString::from(vec![
                (0.0, 0.0),   // 좌하단
                (1.0, 0.0),   // 우하단
                (1.0, 1.0),   // 우상단
                (0.0, 1.0),   // 좌상단
                (0.0, 0.0),   // 닫기
            ]),
            vec![],
        );

        // NUMA 기능이 활성화된 상태에서 polygon_to_cells 호출
        let cells = h3on::geom::polygon_to_cells(
            &polygon,
            Resolution::Nine,
            ContainmentMode::Covers,
        );

        // 결과 검증
        assert!(!cells.is_empty(), "셀이 생성되어야 합니다");
        println!("✅ NUMA 통합 테스트 성공: {} 개의 셀 생성", cells.len());
    }

    #[test]
    fn test_numa_feature_detection() {
        // NUMA feature가 활성화되어 있는지 확인
        assert!(cfg!(feature = "numa"), "NUMA feature가 활성화되어야 합니다");
        assert!(cfg!(feature = "geo"), "geo feature가 활성화되어야 합니다");
        println!("✅ NUMA feature 감지 성공");
    }
}

#[cfg(not(feature = "numa"))]
mod numa_disabled_tests {
    #[test]
    fn test_numa_feature_disabled() {
        // NUMA feature가 비활성화된 경우
        assert!(!cfg!(feature = "numa"), "NUMA feature가 비활성화되어야 합니다");
        println!("✅ NUMA feature 비활성화 상태 확인");
    }
}
