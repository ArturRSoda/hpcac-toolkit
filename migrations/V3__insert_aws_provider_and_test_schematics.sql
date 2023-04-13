insert into public.providers (alias, schematics_table) values ('aws', 'aws_clusters_schematics');
insert into public.aws_clusters_schematics (alias, description, az, master_ami, master_flavor, master_ebs, spot_cluster, worker_count, workers_ami, workers_flavor, workers_ebs, nfs_support, criu_support, blcr_support, ulfm_support) values ('nano-spot-test', 'cheap two-node t2.nano spot cluster for testing', 'us-east-1a', 'ami-08e4e35cccc6189f4', 't2.nano', 10, true, 1, 'ami-08e4e35cccc6189f4', 't2.nano', 10, true, false, false, false);
insert into public.aws_clusters_schematics (alias, description, az, master_ami, master_flavor, master_ebs, spot_cluster, worker_count, workers_ami, workers_flavor, workers_ebs, nfs_support, criu_support, blcr_support, ulfm_support) values ('nano-test', 'cheap two-node t2.nano cluster for testing', 'us-east-1a', 'ami-08e4e35cccc6189f4', 't2.nano', 10, false, 1, 'ami-08e4e35cccc6189f4', 't2.nano', 10, true, false, false, false);
