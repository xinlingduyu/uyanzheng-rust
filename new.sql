-- MySQL dump 10.13  Distrib 5.7.40, for Linux (x86_64)
--
-- Nakamasa-Ichika Database Schema
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
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;
/*!40101 SET character_set_client = @saved_cs_client */;

--
-- Table structure for table `u_app_function`
--

CREATE TABLE `u_app_function` (
  `id` bigint(20) unsigned NOT NULL AUTO_INCREMENT,
  `name` varchar(64) NOT NULL COMMENT '函数名称',
  `code` text NOT NULL COMMENT 'JavaScript代码',
  `notes` varchar(255) DEFAULT '' COMMENT '备注说明',
  `allow` tinyint(1) DEFAULT 0 COMMENT 'VIP权限要求',
  `fen` int(10) DEFAULT 0 COMMENT '积分消耗',
  `state` enum('y','n') DEFAULT 'y' COMMENT '状态',
  `appid` bigint(20) unsigned NOT NULL COMMENT '应用ID',
  PRIMARY KEY (`id`),
  UNIQUE KEY `name_appid` (`name`, `appid`),
  KEY `appid` (`appid`)
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
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;
/*!40101 SET character_set_client = @saved_cs_client */;

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
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;
/*!40101 SET character_set_client = @saved_cs_client */;

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
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;
/*!40101 SET character_set_client = @saved_cs_client */;

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
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;
/*!40101 SET character_set_client = @saved_cs_client */;

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
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;
/*!40101 SET character_set_client = @saved_cs_client */;

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
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;
/*!40101 SET character_set_client = @saved_cs_client */;

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
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;
/*!40101 SET character_set_client = @saved_cs_client */;

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
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;
/*!40101 SET character_set_client = @saved_cs_client */;

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
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;
/*!40101 SET character_set_client = @saved_cs_client */;

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
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;
/*!40101 SET character_set_client = @saved_cs_client */;

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
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;
/*!40101 SET character_set_client = @saved_cs_client */;

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

/*!40103 SET TIME_ZONE=@OLD_TIME_ZONE */;

/*!40101 SET SQL_MODE=@OLD_SQL_MODE */;
/*!40014 SET FOREIGN_KEY_CHECKS=@OLD_FOREIGN_KEY_CHECKS */;
/*!40014 SET UNIQUE_CHECKS=@OLD_UNIQUE_CHECKS */;
/*!40101 SET CHARACTER_SET_CLIENT=@OLD_CHARACTER_SET_CLIENT */;
/*!40101 SET CHARACTER_SET_RESULTS=@OLD_CHARACTER_SET_RESULTS */;
/*!40101 SET COLLATION_CONNECTION=@OLD_COLLATION_CONNECTION */;
/*!40111 SET SQL_NOTES=@OLD_SQL_NOTES */;

-- Dump completed