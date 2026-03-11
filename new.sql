-- MySQL dump 10.13  Distrib 5.7.40, for Linux (x86_64)
--
-- Host: localhost    Database: uyanzheng
-- ------------------------------------------------------
-- Server version	5.7.40-log

/*!40101 SET @OLD_CHARACTER_SET_CLIENT=@@CHARACTER_SET_CLIENT */;
/*!40101 SET @OLD_CHARACTER_SET_RESULTS=@@CHARACTER_SET_RESULTS */;
/*!40101 SET @OLD_COLLATION_CONNECTION=@@COLLATION_CONNECTION */;
/*!40101 SET NAMES utf8mb4 */;
/*!40103 SET @OLD_TIME_ZONE=@@TIME_ZONE */;
/*!40103 SET TIME_ZONE='+00:00' */;
/*!40014 SET @OLD_UNIQUE_CHECKS=@@UNIQUE_CHECKS, UNIQUE_CHECKS=0 */;
/*!40014 SET @OLD_FOREIGN_KEY_CHECKS=@@FOREIGN_KEY_CHECKS, FOREIGN_KEY_CHECKS=0 */;
/*!40101 SET @OLD_SQL_MODE=@@SQL_MODE, SQL_MODE='NO_AUTO_VALUE_ON_ZERO' */;
/*!40111 SET @OLD_SQL_NOTES=@@SQL_NOTES, SQL_NOTES=0 */;

--
-- Table structure for table `u_admin`
--

DROP TABLE IF EXISTS `u_admin`;
/*!40101 SET @saved_cs_client     = @@character_set_client */;
/*!40101 SET character_set_client = utf8 */;
CREATE TABLE `u_admin` (
  `id` bigint(20) unsigned NOT NULL AUTO_INCREMENT,
  `user` varchar(18) NOT NULL,
  `password` varchar(32) NOT NULL,
  `notes` varchar(64) NOT NULL,
  `avatars` varchar(128) DEFAULT NULL,
  `lockin` tinyint(1) DEFAULT '0',
  `auth` json DEFAULT NULL,
  `state` enum('y','n') DEFAULT 'y',
  `appid` json DEFAULT NULL,
  PRIMARY KEY (`id`),
  UNIQUE KEY `user` (`user`)
) ENGINE=InnoDB AUTO_INCREMENT=2 DEFAULT CHARSET=utf8mb4;
/*!40101 SET character_set_client = @saved_cs_client */;

--
-- Dumping data for table `u_admin`
--

LOCK TABLES `u_admin` WRITE;
/*!40000 ALTER TABLE `u_admin` DISABLE KEYS */;
INSERT INTO `u_admin` VALUES (1,'lingmo1','f855fedbdbaab8eda86242d58da1f44a','超级管理员',NULL,0,NULL,'y',NULL);
/*!40000 ALTER TABLE `u_admin` ENABLE KEYS */;
UNLOCK TABLES;

--
-- Table structure for table `u_app_function`
--

CREATE TABLE u_app_function (
     id bigint(20) unsigned NOT NULL AUTO_INCREMENT,
     name varchar(64) NOT NULL COMMENT '函数名称',
     code text NOT NULL COMMENT 'JavaScript代码',
     notes varchar(255) DEFAULT '' COMMENT '备注说明',
     allow tinyint(1) DEFAULT 0 COMMENT 'VIP权限要求',
     fen int(10) DEFAULT 0 COMMENT '积分消耗',
     state enum('y','n') DEFAULT 'y' COMMENT '状态',
     appid bigint(20) unsigned NOT NULL COMMENT '应用ID',
     PRIMARY KEY (id),
     UNIQUE KEY name_appid (name, appid),
     KEY appid (appid)
   ) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COMMENT='云函数表';
--
-- Table structure for table `u_app`
--

DROP TABLE IF EXISTS `u_app`;
/*!40101 SET @saved_cs_client     = @@character_set_client */;
/*!40101 SET character_set_client = utf8 */;
CREATE TABLE `u_app` (
  `id` bigint(20) unsigned NOT NULL AUTO_INCREMENT,
  `app_key` varchar(32) NOT NULL,
  `app_type` enum('user','kami') NOT NULL,
  `app_name` varchar(64) NOT NULL,
  `app_logo` varchar(64) DEFAULT NULL,
  `app_mode` enum('y','n') DEFAULT 'y',
  `app_state` enum('on','off') DEFAULT 'on',
  `app_off_msg` varchar(255) DEFAULT NULL,
  `reg_state` enum('on','off') DEFAULT 'on',
  `reg_off_msg` varchar(255) DEFAULT NULL,
  `reg_way` enum('phone','email','wordnum') DEFAULT 'email',
  `reg_is_inviter` enum('y','n') DEFAULT 'n',
  `reg_time_sn` int(10) DEFAULT '24',
  `reg_time_ip` int(10) DEFAULT '24',
  `reg_award` enum('vip','fen') DEFAULT 'vip',
  `reg_award_val` bigint(10) DEFAULT '86400',
  `logon_state` enum('on','off') DEFAULT 'on',
  `logon_off_msg` varchar(255) DEFAULT NULL,
  `logon_open_wxconfig` json DEFAULT NULL,
  `logon_open_qqconfig` json DEFAULT NULL,
  `logon_token_exp` int(10) DEFAULT '86400',
  `logon_ban_expire` enum('y','n') DEFAULT 'y',
  `logon_sn_dk` enum('y','n') DEFAULT 'n',
  `logon_sn_num` int(2) DEFAULT '0',
  `logon_sn_over_ban` tinyint(1) DEFAULT '1',
  `login_prevent_brute_force` tinyint(1) DEFAULT '1',
  `logon_sn_unbde_auto` tinyint(1) DEFAULT '0',
  `logon_sn_unbde_type` enum('vip','fen') DEFAULT 'fen',
  `logon_sn_unbde_val` int(10) DEFAULT '100',
  `invitee_award` enum('vip','fen') DEFAULT 'vip',
  `invitee_award_val` int(10) DEFAULT '43200',
  `inviter_award` enum('vip','fen') DEFAULT 'vip',
  `inviter_award_val` int(10) DEFAULT '86400',
  `diary_award` enum('vip','fen') DEFAULT 'fen',
  `diary_award_val` int(10) DEFAULT '100',
  `smtp_state` enum('on','off') DEFAULT 'off',
  `smtp_host` varchar(128) DEFAULT 'smtp.qq.com',
  `smtp_user` varchar(128) DEFAULT NULL,
  `smtp_pass` varchar(128) DEFAULT NULL,
  `smtp_port` int(4) DEFAULT '465',
  `sms_state` enum('on','off') DEFAULT 'off',
  `sms_type` varchar(24) DEFAULT 'jie',
  `sms_config` json DEFAULT NULL,
  `vc_time` int(2) DEFAULT '10',
  `vc_length` int(1) DEFAULT '4',
  `vc_frequency` int(5) DEFAULT '120',
  `vc_maximum` int(2) DEFAULT '10',
  `pay_ali_state` enum('on','off') DEFAULT 'off',
  `pay_ali_type` varchar(24) DEFAULT 'jie',
  `pay_ali_config` json DEFAULT NULL,
  `pay_wx_state` enum('on','off') DEFAULT 'off',
  `pay_wx_type` varchar(24) DEFAULT 'jie',
  `pay_wx_config` json DEFAULT NULL,
  PRIMARY KEY (`id`),
  UNIQUE KEY `app_key` (`app_key`)
) ENGINE=InnoDB AUTO_INCREMENT=1002 DEFAULT CHARSET=utf8mb4;
/*!40101 SET character_set_client = @saved_cs_client */;

--
-- Dumping data for table `u_app`
--

LOCK TABLES `u_app` WRITE;
/*!40000 ALTER TABLE `u_app` DISABLE KEYS */;
INSERT INTO `u_app` VALUES (1000,'ef4ddef5d3861211b92c1b6cca21e317','user','demo',NULL,'y','on',NULL,'on','','wordnum','n',24,24,'vip',86400,'on','','{\"appID\": \"\", \"state\": \"on\", \"appSecret\": \"\"}','{\"appID\": \"\", \"state\": \"on\", \"appKey\": \"\"}',86400,'y','n',0,1,1,0,'fen',100,'vip',43200,'vip',86400,'fen',100,'off','smtp.qq.com',NULL,NULL,465,'off','jie',NULL,10,4,120,10,'on','jie','{\"pid\": \"9494\", \"AccessID\": \"jdjdnd\", \"AccessKey\": \"bbxbd\"}','on','jie','{}'),(1001,'671553d8fa656cc215c297ce292a0006','kami','demo2',NULL,'y','on',NULL,'on',NULL,'email','n',24,24,'vip',86400,'on',NULL,NULL,NULL,86400,'y','n',0,1,1,0,'fen',100,'vip',43200,'vip',86400,'fen',100,'off','smtp.qq.com',NULL,NULL,465,'off','jie',NULL,10,4,120,10,'off','jie',NULL,'off','jie',NULL);
/*!40000 ALTER TABLE `u_app` ENABLE KEYS */;
UNLOCK TABLES;

--
-- Table structure for table `u_app_blocklist`
--

DROP TABLE IF EXISTS `u_app_blocklist`;
/*!40101 SET @saved_cs_client     = @@character_set_client */;
/*!40101 SET character_set_client = utf8 */;
CREATE TABLE `u_app_blocklist` (
  `id` bigint(20) unsigned NOT NULL AUTO_INCREMENT,
  `type` enum('ip','sn') NOT NULL,
  `val` varchar(64) NOT NULL,
  `time` bigint(10) NOT NULL,
  `appid` bigint(20) DEFAULT NULL,
  PRIMARY KEY (`id`),
  UNIQUE KEY `type_val_appid` (`type`,`val`,`appid`),
  KEY `appid` (`appid`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;
/*!40101 SET character_set_client = @saved_cs_client */;

--
-- Dumping data for table `u_app_blocklist`
--

LOCK TABLES `u_app_blocklist` WRITE;
/*!40000 ALTER TABLE `u_app_blocklist` DISABLE KEYS */;
/*!40000 ALTER TABLE `u_app_blocklist` ENABLE KEYS */;
UNLOCK TABLES;

--
-- Table structure for table `u_app_extend`
--

DROP TABLE IF EXISTS `u_app_extend`;
/*!40101 SET @saved_cs_client     = @@character_set_client */;
/*!40101 SET character_set_client = utf8 */;
CREATE TABLE `u_app_extend` (
  `id` bigint(20) unsigned NOT NULL AUTO_INCREMENT,
  `name` varchar(128) NOT NULL,
  `var_key` varchar(64) NOT NULL,
  `var_val` text NOT NULL,
  `appid` bigint(20) DEFAULT NULL,
  PRIMARY KEY (`id`),
  KEY `appid` (`appid`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;
/*!40101 SET character_set_client = @saved_cs_client */;

--
-- Dumping data for table `u_app_extend`
--

LOCK TABLES `u_app_extend` WRITE;
/*!40000 ALTER TABLE `u_app_extend` DISABLE KEYS */;
/*!40000 ALTER TABLE `u_app_extend` ENABLE KEYS */;
UNLOCK TABLES;

--
-- Table structure for table `u_app_mi`
--

DROP TABLE IF EXISTS `u_app_mi`;
/*!40101 SET @saved_cs_client     = @@character_set_client */;
/*!40101 SET character_set_client = utf8 */;
CREATE TABLE `u_app_mi` (
  `id` bigint(20) unsigned NOT NULL AUTO_INCREMENT,
  `name` varchar(64) NOT NULL,
  `type` varchar(24) NOT NULL,
  `config` json NOT NULL,
  `sign` enum('y','n') DEFAULT 'n',
  `time` int(10) DEFAULT '60',
  `appid` bigint(20) DEFAULT NULL,
  PRIMARY KEY (`id`),
  KEY `appid` (`appid`)
) ENGINE=InnoDB AUTO_INCREMENT=5 DEFAULT CHARSET=utf8mb4;
/*!40101 SET character_set_client = @saved_cs_client */;

--
-- Dumping data for table `u_app_mi`
--

LOCK TABLES `u_app_mi` WRITE;
/*!40000 ALTER TABLE `u_app_mi` DISABLE KEYS */;
INSERT INTO `u_app_mi` VALUES (1,'测试','rsa','{\"appPublicKey\": \"xnrnxr\", \"appPrivateKey\": \"，dnxnrnr\", \"servicePublicKey\": \"gjcn\", \"servicePrivateKey\": \"tnhtf\"}','y',60,1000),(2,'测额','rc4','{\"key\": \"jdjdjd\", \"encodeType\": \"base64\"}','y',60,1000),(3,'fxr','des','{\"key\": \"jdjdj\", \"encodeType\": \"base64\"}','y',60,1000),(4,'ssss','aes','{\"key\": \"TiSXNzfqxbr4zv4T\", \"encodeType\": \"base64\"}','y',60,1000);
/*!40000 ALTER TABLE `u_app_mi` ENABLE KEYS */;
UNLOCK TABLES;

--
-- Table structure for table `u_app_notice`
--

DROP TABLE IF EXISTS `u_app_notice`;
/*!40101 SET @saved_cs_client     = @@character_set_client */;
/*!40101 SET character_set_client = utf8 */;
CREATE TABLE `u_app_notice` (
  `id` bigint(20) unsigned NOT NULL AUTO_INCREMENT,
  `aid` bigint(20) NOT NULL,
  `visit` bigint(10) DEFAULT '0',
  `content` text NOT NULL,
  `time` bigint(10) NOT NULL,
  `appid` bigint(20) DEFAULT NULL,
  PRIMARY KEY (`id`),
  KEY `aid` (`aid`),
  KEY `appid` (`appid`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;
/*!40101 SET character_set_client = @saved_cs_client */;

--
-- Dumping data for table `u_app_notice`
--

LOCK TABLES `u_app_notice` WRITE;
/*!40000 ALTER TABLE `u_app_notice` DISABLE KEYS */;
/*!40000 ALTER TABLE `u_app_notice` ENABLE KEYS */;
UNLOCK TABLES;

--
-- Table structure for table `u_app_ver`
--

DROP TABLE IF EXISTS `u_app_ver`;
/*!40101 SET @saved_cs_client     = @@character_set_client */;
/*!40101 SET character_set_client = utf8 */;
CREATE TABLE `u_app_ver` (
  `id` bigint(20) unsigned NOT NULL AUTO_INCREMENT,
  `name` varchar(64) DEFAULT NULL,
  `ver_key` varchar(12) DEFAULT 'default',
  `ver_major` int(3) DEFAULT '1',
  `ver_minor` int(3) DEFAULT '0',
  `ver_patch` int(3) DEFAULT '0',
  `ver_state` enum('on','off') DEFAULT 'on',
  `ver_off_msg` varchar(255) DEFAULT NULL,
  `ver_url` varchar(128) DEFAULT NULL,
  `ver_content` text,
  `mid` bigint(20) DEFAULT NULL,
  `discard` tinyint(1) DEFAULT '0',
  `appid` bigint(20) NOT NULL,
  PRIMARY KEY (`id`),
  UNIQUE KEY `ver_key` (`ver_key`,`ver_major`,`ver_minor`,`ver_patch`,`appid`),
  KEY `mid` (`mid`),
  KEY `appid` (`appid`),
  KEY `ver_key_appid` (`ver_key`,`appid`)
) ENGINE=InnoDB AUTO_INCREMENT=4 DEFAULT CHARSET=utf8mb4;
/*!40101 SET character_set_client = @saved_cs_client */;

--
-- Dumping data for table `u_app_ver`
--

LOCK TABLES `u_app_ver` WRITE;
/*!40000 ALTER TABLE `u_app_ver` DISABLE KEYS */;
INSERT INTO `u_app_ver` VALUES (1,'测试','test',1,1,1,'on','','','',NULL,0,1000),(2,'测试','test',1,1,1,'on','','','',NULL,0,1001);
/*!40000 ALTER TABLE `u_app_ver` ENABLE KEYS */;
UNLOCK TABLES;

--
-- Table structure for table `u_cdk_group`
--

DROP TABLE IF EXISTS `u_cdk_group`;
/*!40101 SET @saved_cs_client     = @@character_set_client */;
/*!40101 SET character_set_client = utf8 */;
CREATE TABLE `u_cdk_group` (
  `id` bigint(20) unsigned NOT NULL AUTO_INCREMENT,
  `name` varchar(64) NOT NULL,
  `val` bigint(10) NOT NULL,
  `type` enum('vip','fen','addsn') NOT NULL,
  `price` float(10,2) DEFAULT NULL,
  `appid` bigint(20) NOT NULL,
  PRIMARY KEY (`id`),
  KEY `appid` (`appid`)
) ENGINE=InnoDB AUTO_INCREMENT=3 DEFAULT CHARSET=utf8mb4;
/*!40101 SET character_set_client = @saved_cs_client */;

--
-- Dumping data for table `u_cdk_group`
--

LOCK TABLES `u_cdk_group` WRITE;
/*!40000 ALTER TABLE `u_cdk_group` DISABLE KEYS */;
INSERT INTO `u_cdk_group` VALUES (1,'11',86400,'vip',1.00,1001),(2,'11',86400,'vip',1.00,1000);
/*!40000 ALTER TABLE `u_cdk_group` ENABLE KEYS */;
UNLOCK TABLES;

--
-- Table structure for table `u_cdk_kami`
--

DROP TABLE IF EXISTS `u_cdk_kami`;
/*!40101 SET @saved_cs_client     = @@character_set_client */;
/*!40101 SET character_set_client = utf8 */;
CREATE TABLE `u_cdk_kami` (
  `id` bigint(20) unsigned NOT NULL AUTO_INCREMENT,
  `gid` bigint(20) NOT NULL,
  `type` enum('vip','fen','addsn') NOT NULL,
  `cdk` varchar(64) NOT NULL,
  `val` bigint(10) NOT NULL,
  `email` varchar(64) DEFAULT NULL,
  `phone` bigint(11) DEFAULT NULL,
  `password` varchar(32) DEFAULT NULL,
  `note` varchar(128) DEFAULT NULL,
  `vip` bigint(10) DEFAULT NULL,
  `fen` bigint(10) DEFAULT NULL,
  `add_role` enum('admin','agent') NOT NULL,
  `add_uid` bigint(20) NOT NULL,
  `add_price` float(10,2) DEFAULT '0.00',
  `add_time` int(10) NOT NULL,
  `add_ip` varchar(15) NOT NULL,
  `use_id` bigint(20) DEFAULT NULL,
  `use_time` bigint(10) DEFAULT NULL,
  `use_ip` varchar(15) DEFAULT NULL,
  `out_state` enum('y','n') DEFAULT 'n',
  `out_time` bigint(10) DEFAULT NULL,
  `ban` bigint(10) DEFAULT NULL,
  `ban_msg` varchar(128) DEFAULT NULL,
  `sn_max` int(2) DEFAULT '0',
  `sn_list` json DEFAULT NULL,
  `appid` bigint(20) NOT NULL,
  PRIMARY KEY (`id`),
  UNIQUE KEY `cdk` (`cdk`,`appid`),
  UNIQUE KEY `phone` (`phone`,`appid`),
  UNIQUE KEY `email` (`email`,`appid`),
  KEY `gid` (`gid`),
  KEY `add_role_uid_appid` (`add_role`,`add_uid`,`appid`),
  KEY `add_uid` (`add_uid`),
  KEY `add_time` (`add_time`),
  KEY `use_id` (`use_id`),
  KEY `out_state` (`out_state`),
  KEY `type` (`type`),
  KEY `add_role` (`add_role`),
  KEY `use_time` (`use_time`),
  KEY `ban` (`ban`),
  KEY `appid` (`appid`)
) ENGINE=InnoDB AUTO_INCREMENT=3 DEFAULT CHARSET=utf8mb4;
/*!40101 SET character_set_client = @saved_cs_client */;

--
-- Dumping data for table `u_cdk_kami`
--

LOCK TABLES `u_cdk_kami` WRITE;
/*!40000 ALTER TABLE `u_cdk_kami` DISABLE KEYS */;
INSERT INTO `u_cdk_kami` VALUES (1,1,'vip','AC4KNWVN496R7',86400,NULL,NULL,'','',NULL,NULL,'admin',1,1.00,1765244458,'183.203.195.138',NULL,NULL,NULL,'n',0,NULL,NULL,0,NULL,1001),(2,1,'vip','WYNGC7Q9G81YX',86400,NULL,NULL,'','',1768632788,0,'admin',1,1.00,1768546377,'120.231.156.218',NULL,1768546388,'1.12.252.32','n',0,0,NULL,0,'[{\"time\": 1768546388, \"udid\": \"lingmo123\"}]',1001);
/*!40000 ALTER TABLE `u_cdk_kami` ENABLE KEYS */;
UNLOCK TABLES;

--
-- Table structure for table `u_cdk_user`
--

DROP TABLE IF EXISTS `u_cdk_user`;
/*!40101 SET @saved_cs_client     = @@character_set_client */;
/*!40101 SET character_set_client = utf8 */;
CREATE TABLE `u_cdk_user` (
  `id` bigint(20) unsigned NOT NULL AUTO_INCREMENT,
  `gid` bigint(20) NOT NULL,
  `type` enum('vip','fen','addsn') NOT NULL,
  `cdk` varchar(64) NOT NULL,
  `val` bigint(10) NOT NULL,
  `note` varchar(64) DEFAULT NULL,
  `use_uid` bigint(20) DEFAULT NULL,
  `use_time` bigint(10) DEFAULT NULL,
  `use_ip` varchar(15) DEFAULT NULL,
  `add_role` enum('admin','agent') NOT NULL,
  `add_uid` bigint(20) DEFAULT NULL,
  `add_price` float(10,2) DEFAULT '0.00',
  `add_time` bigint(10) NOT NULL,
  `add_ip` varchar(15) DEFAULT NULL,
  `out_state` enum('y','n') DEFAULT 'n',
  `out_time` bigint(10) DEFAULT NULL,
  `state` enum('y','n') DEFAULT 'y',
  `appid` bigint(20) NOT NULL,
  PRIMARY KEY (`id`),
  UNIQUE KEY `cdk` (`cdk`,`appid`),
  KEY `add_time` (`add_time`),
  KEY `gid` (`gid`),
  KEY `use_uid` (`use_uid`),
  KEY `use_time` (`use_time`),
  KEY `add_uid` (`add_uid`),
  KEY `out_state` (`out_state`),
  KEY `state` (`state`),
  KEY `appid` (`appid`),
  KEY `type` (`type`),
  KEY `add_role` (`add_role`)
) ENGINE=InnoDB AUTO_INCREMENT=3 DEFAULT CHARSET=utf8mb4;
/*!40101 SET character_set_client = @saved_cs_client */;

--
-- Dumping data for table `u_cdk_user`
--

LOCK TABLES `u_cdk_user` WRITE;
/*!40000 ALTER TABLE `u_cdk_user` DISABLE KEYS */;
INSERT INTO `u_cdk_user` VALUES (1,2,'vip','5XTZ6OH74J1D5',86400,'',NULL,NULL,NULL,'admin',1,1.00,1765548619,'118.80.92.126','y',1765548667,'y',1000),(2,2,'vip','FY4WBTPK07KW9',86400,'',NULL,NULL,NULL,'admin',1,1.00,1770548241,'36.110.36.142','n',0,'y',1000);
/*!40000 ALTER TABLE `u_cdk_user` ENABLE KEYS */;
UNLOCK TABLES;

--
-- Table structure for table `u_fen_event`
--

DROP TABLE IF EXISTS `u_fen_event`;
/*!40101 SET @saved_cs_client     = @@character_set_client */;
/*!40101 SET character_set_client = utf8 */;
CREATE TABLE `u_fen_event` (
  `id` bigint(20) unsigned NOT NULL AUTO_INCREMENT,
  `name` varchar(128) NOT NULL,
  `fen` bigint(10) DEFAULT '0',
  `vip` bigint(10) DEFAULT '0',
  `vip_free` enum('y','n') DEFAULT 'n',
  `state` enum('on','off') DEFAULT 'on',
  `appid` bigint(20) NOT NULL,
  PRIMARY KEY (`id`),
  KEY `appid` (`appid`)
) ENGINE=InnoDB AUTO_INCREMENT=2 DEFAULT CHARSET=utf8mb4;
/*!40101 SET character_set_client = @saved_cs_client */;

--
-- Dumping data for table `u_fen_event`
--

LOCK TABLES `u_fen_event` WRITE;
/*!40000 ALTER TABLE `u_fen_event` DISABLE KEYS */;
INSERT INTO `u_fen_event` VALUES (1,'测试',100,0,'n','on',1000);
/*!40000 ALTER TABLE `u_fen_event` ENABLE KEYS */;
UNLOCK TABLES;

--
-- Table structure for table `u_fen_order`
--

DROP TABLE IF EXISTS `u_fen_order`;
/*!40101 SET @saved_cs_client     = @@character_set_client */;
/*!40101 SET character_set_client = utf8 */;
CREATE TABLE `u_fen_order` (
  `id` bigint(20) unsigned NOT NULL AUTO_INCREMENT,
  `fid` bigint(20) NOT NULL,
  `uid` bigint(20) NOT NULL,
  `name` varchar(128) NOT NULL,
  `mark` varchar(255) DEFAULT NULL,
  `fen` bigint(10) DEFAULT '0',
  `vip` bigint(10) DEFAULT '0',
  `time` bigint(10) NOT NULL,
  `appid` bigint(20) NOT NULL,
  PRIMARY KEY (`id`),
  KEY `uid` (`uid`),
  KEY `mark` (`mark`),
  KEY `appid` (`appid`),
  KEY `fid` (`fid`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;
/*!40101 SET character_set_client = @saved_cs_client */;

--
-- Dumping data for table `u_fen_order`
--

LOCK TABLES `u_fen_order` WRITE;
/*!40000 ALTER TABLE `u_fen_order` DISABLE KEYS */;
/*!40000 ALTER TABLE `u_fen_order` ENABLE KEYS */;
UNLOCK TABLES;

--
-- Table structure for table `u_goods`
--

DROP TABLE IF EXISTS `u_goods`;
/*!40101 SET @saved_cs_client     = @@character_set_client */;
/*!40101 SET character_set_client = utf8 */;
CREATE TABLE `u_goods` (
  `id` bigint(20) unsigned NOT NULL AUTO_INCREMENT,
  `name` varchar(128) NOT NULL,
  `type` enum('vip','fen','agent','addsn') NOT NULL,
  `val` bigint(20) NOT NULL,
  `money` float(10,2) NOT NULL,
  `blurb` text,
  `state` enum('y','n') DEFAULT 'y',
  `appid` bigint(20) NOT NULL,
  PRIMARY KEY (`id`),
  KEY `appid` (`appid`)
) ENGINE=InnoDB AUTO_INCREMENT=2 DEFAULT CHARSET=utf8mb4;
/*!40101 SET character_set_client = @saved_cs_client */;

--
-- Dumping data for table `u_goods`
--

LOCK TABLES `u_goods` WRITE;
/*!40000 ALTER TABLE `u_goods` DISABLE KEYS */;
INSERT INTO `u_goods` VALUES (1,'会员','vip',86400,1.00,NULL,'y',1000);
/*!40000 ALTER TABLE `u_goods` ENABLE KEYS */;
UNLOCK TABLES;

--
-- Table structure for table `u_login`
--

DROP TABLE IF EXISTS `u_login`;
/*!40101 SET @saved_cs_client     = @@character_set_client */;
/*!40101 SET character_set_client = utf8 */;
CREATE TABLE `u_login` (
  `id` bigint(20) unsigned NOT NULL AUTO_INCREMENT,
  `uid` bigint(20) NOT NULL,
  `token` varchar(32) NOT NULL,
  `sn` varchar(128) DEFAULT NULL,
  `ip` varchar(15) DEFAULT NULL,
  `time` bigint(10) NOT NULL,
  `appid` bigint(20) NOT NULL,
  PRIMARY KEY (`id`),
  UNIQUE KEY `token` (`token`),
  KEY `uid` (`uid`),
  KEY `sn` (`sn`),
  KEY `appid` (`appid`)
) ENGINE=InnoDB AUTO_INCREMENT=3 DEFAULT CHARSET=utf8mb4;
/*!40101 SET character_set_client = @saved_cs_client */;

--
-- Dumping data for table `u_login`
--

LOCK TABLES `u_login` WRITE;
/*!40000 ALTER TABLE `u_login` DISABLE KEYS */;
/*!40000 ALTER TABLE `u_login` ENABLE KEYS */;
UNLOCK TABLES;

--
-- Table structure for table `u_logs`
--

DROP TABLE IF EXISTS `u_logs`;
/*!40101 SET @saved_cs_client     = @@character_set_client */;
/*!40101 SET character_set_client = utf8 */;
CREATE TABLE `u_logs` (
  `id` bigint(20) unsigned NOT NULL AUTO_INCREMENT,
  `ug` enum('admin','agent','user','kami') NOT NULL,
  `uid` bigint(20) NOT NULL,
  `type` varchar(64) NOT NULL,
  `details` json DEFAULT NULL,
  `time` bigint(10) NOT NULL,
  `ip` varchar(15) NOT NULL,
  `ip_address` varchar(128) DEFAULT NULL,
  `appid` bigint(20) DEFAULT NULL,
  PRIMARY KEY (`id`),
  KEY `appid` (`appid`),
  KEY `ug` (`ug`),
  KEY `uid` (`uid`),
  KEY `uid_type_appid` (`uid`,`type`,`appid`),
  KEY `type` (`type`)
) ENGINE=InnoDB AUTO_INCREMENT=41 DEFAULT CHARSET=utf8mb4;
/*!40101 SET character_set_client = @saved_cs_client */;

--
-- Dumping data for table `u_logs`
--

LOCK TABLES `u_logs` WRITE;
/*!40000 ALTER TABLE `u_logs` DISABLE KEYS */;
INSERT INTO `u_logs` VALUES (1,'admin',1,'app_add',NULL,1765197454,'120.231.157.7','广东清远',1000),(2,'admin',1,'app_add',NULL,1765244426,'183.203.195.138','山西阳泉',1001),(3,'admin',1,'cdkGroup_add',NULL,1765244444,'183.203.195.138','山西阳泉',1001),(4,'admin',1,'cdkGroup_add','{\"id\": 1, \"val\": 86400, \"name\": \"11\", \"type\": \"vip\", \"appid\": 1001, \"price\": 1}',1765244450,'183.203.195.138','山西阳泉',1001),(5,'admin',1,'cdkKami_add','{\"gid\": 1, \"num\": 1, \"out\": \"\", \"pre\": \"\", \"note\": \"\", \"length\": 13, \"password\": \"\"}',1765244458,'183.203.195.138','山西阳泉',1001),(6,'admin',1,'cdkGroup_add',NULL,1765548606,'118.80.92.126','山西阳泉',1000),(7,'admin',1,'cdkGroup_add','{\"id\": 2, \"val\": 86400, \"name\": \"11\", \"type\": \"vip\", \"appid\": 1000, \"price\": 1}',1765548611,'118.80.92.126','山西阳泉',1000),(8,'admin',1,'cdkUser_add','{\"gid\": 2, \"num\": 1, \"out\": \"\", \"pre\": \"\", \"note\": \"\", \"length\": 13}',1765548619,'118.80.92.126','山西阳泉',1000),(9,'admin',1,'cdkUser_outall','{\"ids\": [1], \"out\": \"txt\"}',1765548628,'118.80.92.126','山西阳泉',1000),(10,'admin',1,'cdkUser_outall','{\"ids\": [1], \"out\": \"txt\"}',1765548667,'118.80.92.126','山西阳泉',1000),(11,'admin',1,'user_add',NULL,1768546127,'120.231.156.218','广东清远',1000),(12,'admin',1,'user_edit','{\"id\": 1, \"fen\": 0, \"vip\": 86398}',1768546137,'120.231.156.218','广东清远',1000),(13,'admin',1,'ver_add','{\"mid\": 0, \"name\": \"测试\", \"ver_key\": \"test\", \"ver_major\": 1, \"ver_minor\": 1, \"ver_patch\": 1, \"ver_state\": \"on\", \"ver_new_url\": \"\", \"ver_off_msg\": \"\", \"ver_new_content\": \"\"}',1768546205,'120.231.156.218','广东清远',1000),(14,'admin',1,'app_edit','{\"reg_way\": \"wordnum\", \"reg_state\": \"on\", \"logon_sn_dk\": \"n\", \"logon_state\": \"on\", \"reg_off_msg\": \"\", \"reg_time_ip\": 24, \"reg_time_sn\": 24, \"logon_sn_num\": 0, \"logon_off_msg\": \"\", \"reg_is_inviter\": \"n\", \"logon_token_exp\": 86400, \"logon_ban_expire\": \"y\", \"logon_sn_over_ban\": true, \"logon_sn_unbde_val\": 100, \"logon_open_qqconfig\": {\"appID\": \"\", \"state\": \"on\", \"appKey\": \"\"}, \"logon_open_wxconfig\": {\"appID\": \"\", \"state\": \"on\", \"appSecret\": \"\"}, \"logon_sn_unbde_auto\": false, \"logon_sn_unbde_type\": \"fen\", \"login_prevent_brute_force\": true}',1768546267,'120.231.156.218','广东清远',1000),(15,'user',1,'login',NULL,1768546283,'1.12.252.32','广东广州',1000),(16,'admin',1,'ver_add','{\"mid\": 0, \"name\": \"测试\", \"ver_key\": \"test\", \"ver_major\": 1, \"ver_minor\": 1, \"ver_patch\": 1, \"ver_state\": \"on\", \"ver_new_url\": \"\", \"ver_off_msg\": \"\", \"ver_new_content\": \"\"}',1768546359,'120.231.156.218','广东清远',1001),(17,'admin',1,'cdkKami_add','{\"gid\": 1, \"num\": 1, \"out\": \"\", \"pre\": \"\", \"note\": \"\", \"length\": 13, \"password\": \"\"}',1768546377,'120.231.156.218','广东清远',1001),(18,'kami',2,'login',NULL,1768546388,'1.12.252.32','广东广州',1001),(19,'kami',2,'info',NULL,1768546451,'1.12.252.32','广东广州',1001),(20,'admin',1,'user_add',NULL,1770280737,'36.110.36.142','北京北京',1000),(21,'admin',1,'app_edit','{\"pay_wx_type\": \"jie\", \"pay_ali_type\": \"jie\", \"pay_wx_state\": \"off\", \"pay_ali_state\": \"on\", \"pay_wx_config\": {}, \"pay_ali_config\": {}}',1770360560,'36.110.36.142','北京北京',1000),(22,'admin',1,'app_edit','{\"pay_wx_type\": \"jie\", \"pay_ali_type\": \"jie\", \"pay_wx_state\": \"off\", \"pay_ali_state\": \"off\", \"pay_wx_config\": {}, \"pay_ali_config\": {}}',1770439467,'36.110.36.142','北京北京',1000),(23,'admin',1,'app_edit','{\"pay_wx_type\": \"jie\", \"pay_ali_type\": \"jie\", \"pay_wx_state\": \"on\", \"pay_ali_state\": \"on\", \"pay_wx_config\": {}, \"pay_ali_config\": {}}',1770439486,'36.110.36.142','北京北京',1000),(24,'admin',1,'app_edit','{\"pay_wx_type\": \"jie\", \"pay_ali_type\": \"jie\", \"pay_wx_state\": \"on\", \"pay_ali_state\": \"on\", \"pay_wx_config\": {}, \"pay_ali_config\": {\"pid\": \"9494\", \"AccessID\": \"jdjdnd\", \"AccessKey\": \"bbxbd\"}}',1770443273,'36.110.36.142','北京北京',1000),(25,'admin',1,'cdkUser_add',NULL,1770548235,'36.110.36.142','北京北京',1000),(26,'admin',1,'cdkUser_add','{\"gid\": 2, \"num\": 1, \"out\": \"\", \"pre\": \"\", \"note\": \"\", \"length\": 13}',1770548241,'36.110.36.142','北京北京',1000),(27,'admin',1,'ver_add',NULL,1770628301,'36.110.36.142',NULL,1000),(28,'admin',1,'ver_add',NULL,1770628308,'36.110.36.142',NULL,1000),(29,'admin',1,'ver_add',NULL,1770628316,'36.110.36.142',NULL,1000),(30,'admin',1,'ver_add',NULL,1770628340,'36.110.36.142',NULL,1000),(31,'admin',1,'ver_add','{\"mid\": 0, \"name\": \"cce\", \"ver_key\": \"ce\", \"ver_major\": 1, \"ver_minor\": 0, \"ver_patch\": 0, \"ver_state\": \"on\", \"ver_new_url\": \"\", \"ver_off_msg\": \"\", \"ver_new_content\": \"\"}',1770628346,'36.110.36.142',NULL,1000),(32,'admin',1,'ver_del','{\"id\": 3}',1770628361,'36.110.36.142',NULL,1000),(33,'admin',1,'encryption_add',NULL,1770684301,'36.110.36.142','北京北京',1000),(34,'admin',1,'encryption_add',NULL,1770684326,'36.110.36.142','北京北京',1000),(35,'admin',1,'encryption_add',NULL,1770684332,'36.110.36.142','北京北京',1000),(36,'admin',1,'encryption_add',NULL,1770684368,'36.110.36.142','北京北京',1000),(37,'admin',1,'encryption_add',NULL,1770684383,'36.110.36.142','北京北京',1000),(38,'admin',1,'goods_add',NULL,1770716274,'36.110.36.142','北京北京',1000),(39,'admin',1,'goods_add','{\"val\": 86400, \"name\": \"会员\", \"type\": \"vip\", \"blurb\": \"\", \"money\": 1}',1770716279,'36.110.36.142','北京北京',1000),(40,'admin',1,'fenEvent_add',NULL,1770775630,'36.110.36.142','北京北京',1000);
/*!40000 ALTER TABLE `u_logs` ENABLE KEYS */;
UNLOCK TABLES;

--
-- Table structure for table `u_message`
--

DROP TABLE IF EXISTS `u_message`;
/*!40101 SET @saved_cs_client     = @@character_set_client */;
/*!40101 SET character_set_client = utf8 */;
CREATE TABLE `u_message` (
  `id` bigint(20) unsigned NOT NULL AUTO_INCREMENT,
  `uid` bigint(20) NOT NULL,
  `utype` enum('user','admin') NOT NULL,
  `title` varchar(128) DEFAULT NULL,
  `content` text NOT NULL,
  `reply_id` bigint(20) DEFAULT NULL,
  `file` json DEFAULT NULL,
  `time` bigint(10) NOT NULL,
  `last_time` bigint(10) DEFAULT NULL,
  `state` int(1) DEFAULT '0',
  `appid` bigint(20) NOT NULL,
  PRIMARY KEY (`id`),
  KEY `utype` (`utype`),
  KEY `title` (`title`),
  KEY `reply_id` (`reply_id`),
  KEY `state` (`state`),
  KEY `appid` (`appid`),
  KEY `uid` (`uid`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;
/*!40101 SET character_set_client = @saved_cs_client */;

--
-- Dumping data for table `u_message`
--

LOCK TABLES `u_message` WRITE;
/*!40000 ALTER TABLE `u_message` DISABLE KEYS */;
/*!40000 ALTER TABLE `u_message` ENABLE KEYS */;
UNLOCK TABLES;

--
-- Table structure for table `u_order`
--

DROP TABLE IF EXISTS `u_order`;
/*!40101 SET @saved_cs_client     = @@character_set_client */;
/*!40101 SET character_set_client = utf8 */;
CREATE TABLE `u_order` (
  `id` bigint(20) unsigned NOT NULL AUTO_INCREMENT,
  `uid` bigint(20) NOT NULL,
  `gid` bigint(20) NOT NULL,
  `inviter_id` bigint(20) DEFAULT NULL,
  `order_no` varchar(40) NOT NULL,
  `trade_no` varchar(60) DEFAULT NULL,
  `name` varchar(128) NOT NULL,
  `money` float(10,2) NOT NULL,
  `divide_money` float(10,2) DEFAULT NULL,
  `type` varchar(12) NOT NULL,
  `val` bigint(20) NOT NULL,
  `payment` enum('ali','wx') DEFAULT NULL,
  `add_time` bigint(10) NOT NULL,
  `end_time` bigint(10) DEFAULT NULL,
  `state` int(1) DEFAULT '0',
  `appid` bigint(20) NOT NULL,
  PRIMARY KEY (`id`),
  KEY `inviter_id` (`inviter_id`),
  KEY `type` (`type`),
  KEY `ptype` (`payment`),
  KEY `appid` (`state`,`appid`),
  KEY `uid` (`uid`),
  KEY `gid` (`gid`),
  KEY `order_no` (`order_no`),
  KEY `trade_no` (`trade_no`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;
/*!40101 SET character_set_client = @saved_cs_client */;

--
-- Dumping data for table `u_order`
--

LOCK TABLES `u_order` WRITE;
/*!40000 ALTER TABLE `u_order` DISABLE KEYS */;
/*!40000 ALTER TABLE `u_order` ENABLE KEYS */;
UNLOCK TABLES;

--
-- Table structure for table `u_user`
--

DROP TABLE IF EXISTS `u_user`;
/*!40101 SET @saved_cs_client     = @@character_set_client */;
/*!40101 SET character_set_client = utf8 */;
CREATE TABLE `u_user` (
  `id` bigint(20) unsigned NOT NULL AUTO_INCREMENT,
  `email` varchar(64) DEFAULT NULL,
  `phone` bigint(11) DEFAULT NULL,
  `acctno` varchar(18) DEFAULT NULL,
  `nickname` varchar(128) DEFAULT NULL,
  `avatars` varchar(255) DEFAULT NULL,
  `password` varchar(32) NOT NULL,
  `inviter_id` bigint(20) DEFAULT NULL,
  `vip` bigint(10) DEFAULT NULL,
  `fen` bigint(10) DEFAULT '0',
  `extend` json DEFAULT NULL,
  `open_wx` varchar(128) DEFAULT NULL,
  `open_qq` varchar(128) DEFAULT NULL,
  `reg_time` bigint(10) NOT NULL,
  `reg_ip` varchar(15) NOT NULL,
  `reg_sn` varchar(64) DEFAULT NULL,
  `sn_list` json DEFAULT NULL,
  `sn_max` bigint(2) DEFAULT '0',
  `ban` bigint(10) DEFAULT NULL,
  `ban_msg` varchar(255) DEFAULT NULL,
  `appid` bigint(20) NOT NULL,
  PRIMARY KEY (`id`),
  UNIQUE KEY `email_appid` (`email`,`appid`),
  UNIQUE KEY `phone_appid` (`phone`,`appid`),
  UNIQUE KEY `acctno_appid` (`acctno`,`appid`),
  UNIQUE KEY `open_wx_appid` (`open_wx`,`appid`),
  UNIQUE KEY `open_qq_appid` (`open_qq`,`appid`),
  KEY `appid` (`appid`),
  KEY `inviter_id` (`inviter_id`),
  KEY `reg_ip` (`reg_ip`),
  KEY `ban` (`ban`),
  KEY `reg_sn` (`reg_sn`)
) ENGINE=InnoDB AUTO_INCREMENT=3 DEFAULT CHARSET=utf8mb4;
/*!40101 SET character_set_client = @saved_cs_client */;

--
-- Dumping data for table `u_user`
--

LOCK TABLES `u_user` WRITE;
/*!40000 ALTER TABLE `u_user` DISABLE KEYS */;
INSERT INTO `u_user` VALUES (1,NULL,NULL,'test123',NULL,NULL,'e10adc3949ba59abbe56e057f20f883e',NULL,1768632535,0,NULL,NULL,NULL,1768546127,'120.231.156.218',NULL,NULL,0,NULL,NULL,1000),(2,NULL,NULL,'46465959',NULL,NULL,'5c5aa8549d7ee3356120dbbe3f0ec3f5',NULL,NULL,0,NULL,NULL,NULL,1770280737,'36.110.36.142',NULL,NULL,0,NULL,NULL,1000);
/*!40000 ALTER TABLE `u_user` ENABLE KEYS */;
UNLOCK TABLES;

--
-- Table structure for table `u_vcode`
--

DROP TABLE IF EXISTS `u_vcode`;
/*!40101 SET @saved_cs_client     = @@character_set_client */;
/*!40101 SET character_set_client = utf8 */;
CREATE TABLE `u_vcode` (
  `id` bigint(20) unsigned NOT NULL AUTO_INCREMENT,
  `eorp` varchar(64) NOT NULL,
  `type` varchar(12) NOT NULL,
  `code` int(6) NOT NULL,
  `usable` enum('y','n') DEFAULT 'y',
  `time` bigint(10) NOT NULL,
  `ip` varchar(15) NOT NULL,
  `appid` bigint(20) NOT NULL,
  PRIMARY KEY (`id`),
  KEY `eorp` (`eorp`),
  KEY `type` (`type`),
  KEY `code` (`code`),
  KEY `appid` (`appid`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;
/*!40101 SET character_set_client = @saved_cs_client */;

--
-- Dumping data for table `u_vcode`
--

LOCK TABLES `u_vcode` WRITE;
/*!40000 ALTER TABLE `u_vcode` DISABLE KEYS */;
/*!40000 ALTER TABLE `u_vcode` ENABLE KEYS */;
UNLOCK TABLES;

--
-- Dumping events for database 'uyanzheng'
--

--
-- Dumping routines for database 'uyanzheng'
--
/*!40103 SET TIME_ZONE=@OLD_TIME_ZONE */;

/*!40101 SET SQL_MODE=@OLD_SQL_MODE */;
/*!40014 SET FOREIGN_KEY_CHECKS=@OLD_FOREIGN_KEY_CHECKS */;
/*!40014 SET UNIQUE_CHECKS=@OLD_UNIQUE_CHECKS */;
/*!40101 SET CHARACTER_SET_CLIENT=@OLD_CHARACTER_SET_CLIENT */;
/*!40101 SET CHARACTER_SET_RESULTS=@OLD_CHARACTER_SET_RESULTS */;
/*!40101 SET COLLATION_CONNECTION=@OLD_COLLATION_CONNECTION */;
/*!40111 SET SQL_NOTES=@OLD_SQL_NOTES */;

-- Dump completed on 2026-02-11 12:55:45
