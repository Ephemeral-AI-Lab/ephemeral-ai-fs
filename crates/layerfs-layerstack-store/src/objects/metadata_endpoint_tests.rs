use super::*;
use rusqlite::StatementStatus;

const FULL_MAX: &str = "SELECT COALESCE(MAX(first_ordinal+count),1) FROM metadata_value_groups";

fn catalogue() -> Connection {
    let connection = Connection::open_in_memory().unwrap();
    connection
        .execute_batch(include_str!("../../sql/schema/v10.sql"))
        .unwrap();
    connection
}

// Endpoint-only rows satisfy the actual schema; payload authentication is covered
// by the existing admission corruption/reopen/rollback tests.
fn insert(connection: &Connection, first: i64, count: i64) {
    connection
        .execute(
            "INSERT OR IGNORE INTO object_packs(pack_id,data) VALUES (?1,x'00')",
            [first],
        )
        .unwrap();
    connection.execute(
        "INSERT INTO metadata_value_groups(first_ordinal,count,pack_id,group_number,digest) VALUES (?1,?2,?1,0,zeroblob(32))",
        [first,count],
    ).unwrap();
}

fn query(connection: &Connection, sql: &str) -> (u64, [i32; 4]) {
    let mut statement = connection.prepare(sql).unwrap();
    let result: i64 = if statement.parameter_count() == 0 {
        statement.query_row([], |row| row.get(0)).unwrap()
    } else {
        statement
            .query_row([VALUES_PER_GROUP as i64], |row| row.get(0))
            .unwrap()
    };
    let counters = [
        StatementStatus::VmStep,
        StatementStatus::FullscanStep,
        StatementStatus::Sort,
        StatementStatus::MemUsed,
    ]
    .map(|status| statement.get_status(status));
    (result as u64, counters)
}

#[test]
fn metadata_endpoint_matches_full_max_with_gaps_overlaps_and_boundaries() {
    let mut connection = catalogue();
    assert_eq!(next_ordinal(&connection).unwrap(), 1);
    for (first, count) in [(1, 165), (2, 1), (166, 165), (330, 1), (331, 1)] {
        insert(&connection, first, count);
        assert_eq!(
            next_ordinal(&connection).unwrap(),
            query(&connection, FULL_MAX).0
        );
    }
    // The second row has the largest key but a smaller end: a plain tail query
    // would return 3 after the first two inserts instead of the correct 166.
    connection
        .execute("DELETE FROM metadata_value_groups", [])
        .unwrap();
    for first in 1..=165 {
        insert(&connection, first, 166 - first);
    }
    assert_eq!(next_ordinal(&connection).unwrap(), 166);
    {
        let transaction = connection.transaction().unwrap();
        insert(&transaction, i64::from(u32::MAX), 1);
        assert_eq!(next_ordinal(&transaction).unwrap(), u64::from(u32::MAX) + 1);
        transaction.rollback().unwrap();
    }
    assert_eq!(next_ordinal(&connection).unwrap(), 166);
    for count in [0, 166] {
        assert!(connection
            .execute(
                "UPDATE metadata_value_groups SET count=?1 WHERE first_ordinal=1",
                [count]
            )
            .is_err());
    }
    // Altered but schema-conforming counts still produce exactly the old answer.
    connection
        .execute(
            "UPDATE metadata_value_groups SET count=165 WHERE first_ordinal=100",
            [],
        )
        .unwrap();
    assert_eq!(next_ordinal(&connection).unwrap(), 265);
    assert_eq!(
        next_ordinal(&connection).unwrap(),
        query(&connection, FULL_MAX).0
    );
}

#[test]
fn metadata_endpoint_work_is_bounded_on_product_sqlite() {
    let mut connection = catalogue();
    println!("endpoint SQLite {}", rusqlite::version());
    let mut previous = 0;
    for groups in [100, 1000, 10000, 100000] {
        let transaction = connection.transaction().unwrap();
        for group in previous..groups {
            insert(&transaction, group * 165 + 1, 165);
        }
        transaction.commit().unwrap();
        previous = groups;
        let old = query(&connection, FULL_MAX);
        let new = query(&connection, NEXT_ORDINAL_SQL);
        println!("endpoint groups={groups} old={old:?} bounded={new:?}");
        assert_eq!(old.0, new.0);
        assert_eq!(new.0, (groups * 165 + 1) as u64);
        assert!(old.1[0] > groups as i32);
        assert!(new.1[0] <= 1200, "endpoint must not traverse history");
        assert_eq!(new.1[1], 0, "no full scan");
        assert_eq!(new.1[2], 0, "no temporary sort");
        assert!(new.1[3] <= 64 * 1024, "bounded statement memory");
    }
    connection
        .execute("DELETE FROM metadata_value_groups", [])
        .unwrap();
    for first in 1..=1000 {
        insert(&connection, first, 165);
    }
    let old = query(&connection, FULL_MAX);
    let new = query(&connection, NEXT_ORDINAL_SQL);
    println!("endpoint dense-tail old={old:?} bounded={new:?}");
    let mut statement = connection
        .prepare(&format!("EXPLAIN {NEXT_ORDINAL_SQL}"))
        .unwrap();
    let opcodes = statement
        .query_map([VALUES_PER_GROUP as i64], |row| row.get::<_, String>(1))
        .unwrap()
        .collect::<rusqlite::Result<Vec<_>>>()
        .unwrap();
    println!("endpoint opcodes={opcodes:?}");
    assert!(opcodes.iter().any(|opcode| opcode == "Last"));
    assert!(opcodes.iter().any(|opcode| opcode == "SeekGT"));
    assert!(!opcodes
        .iter()
        .any(|opcode| opcode == "SorterOpen" || opcode == "OpenEphemeral"));
    assert_eq!(old.0, new.0);
    assert!(new.1[0] <= 1200);
    assert_eq!((new.1[1], new.1[2]), (0, 0));
    assert!(new.1[3] <= 64 * 1024);
}
