use rocksdb::{
    BlockBasedIndexType, BlockBasedOptions, ColumnFamilyDescriptor, IteratorMode, Options,
    ReadOptions, SliceTransform, TransactionDB, TransactionDBOptions, TransactionOptions,
};

enum ArangoRocksDBColumnFamily {
    Definitions(ColumnFamilyDescriptor),
    Documents(ColumnFamilyDescriptor),
    PrimaryIndex(ColumnFamilyDescriptor),
    EdgeIndex(ColumnFamilyDescriptor),
    VPackIndex(ColumnFamilyDescriptor),
    GeoIndex(ColumnFamilyDescriptor),
    FulltextIndex(ColumnFamilyDescriptor),
    ReplicatedLogs(ColumnFamilyDescriptor),
    ZkdIndex(ColumnFamilyDescriptor),
    Invalid(ColumnFamilyDescriptor),
}

impl ArangoRocksDBColumnFamily {
    fn definitions() -> ColumnFamilyDescriptor {
        let opts = Options::default();
        ColumnFamilyDescriptor::new("default", opts)
    }
    fn documents() -> ColumnFamilyDescriptor {
        let mut opts = Options::default();
        opts.set_optimize_filters_for_hits(true);
        ColumnFamilyDescriptor::new("Documents", opts)
    }
    fn primary_index() -> ColumnFamilyDescriptor {
        let mut opts = Options::default();
        opts.set_optimize_filters_for_hits(true);
        opts.set_prefix_extractor(SliceTransform::create_fixed_prefix(std::mem::size_of::<
            usize,
        >()));
        ColumnFamilyDescriptor::new("PrimaryIndex", opts)
    }
    fn edge_index() -> ColumnFamilyDescriptor {
        let mut opts = Options::default();
        opts.set_optimize_filters_for_hits(true);
        opts.set_prefix_extractor(SliceTransform::create(
            "RocksDBPrefixExtractor",
            |x: &[u8]| {
                if x.last() != Some(&0u8) {
                    &x[0..x.len() - 9]
                } else {
                    x
                }
            },
            Some(|x: &[u8]| x.last() != Some(&0u8)),
        ));

        let mut tableopts = BlockBasedOptions::default();
        tableopts.set_index_type(BlockBasedIndexType::HashSearch);

        opts.set_block_based_table_factory(&tableopts);
        ColumnFamilyDescriptor::new("EdgeIndex", opts)
    }
    fn vpack_index() -> ColumnFamilyDescriptor {
        let mut opts = Options::default();
        opts.set_optimize_filters_for_hits(true);
        opts.set_prefix_extractor(SliceTransform::create_fixed_prefix(std::mem::size_of::<
            usize,
        >()));
        opts.set_comparator("RocksDBVPackComparator", |x: &[u8], u: &[u8]| {
            std::cmp::Ordering::Equal
        });

        let mut tableopts = BlockBasedOptions::default();
        // tableopgs reset filter opts
        tableopts.set_index_type(BlockBasedIndexType::HashSearch);

        ColumnFamilyDescriptor::new("VPackIndex", opts)
    }
    fn geo_index() -> ColumnFamilyDescriptor {
        let mut opts = Options::default();
        opts.set_optimize_filters_for_hits(true);
        opts.set_prefix_extractor(SliceTransform::create_fixed_prefix(std::mem::size_of::<
            usize,
        >()));
        ColumnFamilyDescriptor::new("GeoIndex", opts)
    }
    fn fulltext_index() -> ColumnFamilyDescriptor {
        let mut opts = Options::default();
        opts.set_prefix_extractor(SliceTransform::create_fixed_prefix(std::mem::size_of::<
            usize,
        >()));
        opts.set_optimize_filters_for_hits(true);
        ColumnFamilyDescriptor::new("FulltextIndex", opts)
    }
    fn replicated_logs() -> ColumnFamilyDescriptor {
        let mut opts = Options::default();
        opts.set_optimize_filters_for_hits(true);
        opts.set_prefix_extractor(SliceTransform::create_fixed_prefix(std::mem::size_of::<
            usize,
        >()));
        ColumnFamilyDescriptor::new("ReplicatedLogs", opts)
    }
    fn zkd_index() -> ColumnFamilyDescriptor {
        let mut opts = Options::default();
        opts.set_optimize_filters_for_hits(true);
        opts.set_prefix_extractor(SliceTransform::create_fixed_prefix(std::mem::size_of::<
            usize,
        >()));
        ColumnFamilyDescriptor::new("ZkdIndex", opts)
    }
}
/*
void dump_colllist(rocksdb::TransactionDB* db, std::string const& outfile) {
  rocksdb::WriteOptions opts;
  rocksdb::TransactionOptions topts;
  rocksdb::Transaction* trx = db->BeginTransaction(opts, topts);
  rocksdb::ReadOptions ropts;
  ropts.verify_checksums = false;
  ropts.fill_cache = false;
  std::ofstream out(outfile.c_str(), std::ios::out);
  out << "Dumping collection list directly from documents family:\n";
  std::string startKey;
  rocksdb::Slice start;
  rocksdb::Iterator* it =
      trx->GetIterator(ropts, cfHandles[(size_t)Family::Documents]);
  it->Seek(start);
  std::string line1;
  while (it->Valid()) {
    rocksdb::Slice key = it->key();
    uint64_t id;
    memcpy(&id, key.data(), sizeof(uint64_t));
    id = be64toh(id);
    out << id << " = 0x" << std::hex << id << std::dec << "\n";
    id = htobe64(id + 1);
    startKey.assign((char const*)&id, sizeof(uint64_t));
    rocksdb::Slice sl(startKey);
    it->Seek(sl);
  }
  delete it;
  delete trx;
}
*/

fn dump_collection_list(db: &TransactionDB) -> () {
    let cf_handle = db.cf_handle("Documents").unwrap();
    let mut ropts = ReadOptions::default();
    ropts.set_verify_checksums(false);
    ropts.fill_cache(false);

    let trx = db.transaction();

    let iter = trx.iterator_cf(cf_handle, IteratorMode::Start);
    for item in iter {
        let (key, value) = item.unwrap();
        println!("key: {:?} value: {:?}", key, value);
    }
}

fn main() -> () {
    let path = "/home/makx/scratch/bananas/engine-rocksdb";
    let mut cf_opts = Options::default();
    cf_opts.set_max_write_buffer_number(16);
    let cfs: Vec<ColumnFamilyDescriptor> = vec![
        ArangoRocksDBColumnFamily::definitions(),
        ArangoRocksDBColumnFamily::documents(),
        ArangoRocksDBColumnFamily::primary_index(),
        ArangoRocksDBColumnFamily::edge_index(),
        ArangoRocksDBColumnFamily::vpack_index(),
        ArangoRocksDBColumnFamily::geo_index(),
        ArangoRocksDBColumnFamily::fulltext_index(),
        ArangoRocksDBColumnFamily::replicated_logs(),
        ArangoRocksDBColumnFamily::zkd_index(),
    ];

    let mut db_opts = Options::default();
    db_opts.create_missing_column_families(false);
    db_opts.create_if_missing(false);
    let mut txdb_opts = TransactionDBOptions::default();
    {
        let db = TransactionDB::open_cf_descriptors(&db_opts, &txdb_opts, path, cfs).unwrap();
        dump_collection_list(&db);
    }
    //    let _ = DB::destroy(&db_opts, path);

    println!("Foo blo");
    println!("Hello world");
}
