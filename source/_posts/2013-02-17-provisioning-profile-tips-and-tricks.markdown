---
layout: post
title: "Provisioning Profile Tips &amp; Tricks"
date: 2012-12-10 00:42
comments: true
categories: [iOS, Provisioning]
---

Use this command to verify a .mobileprovision file and output its contents.
```
openssl smime -in /path/to/your.mobileprovision -inform der -verify
```
If you are on Linux you may not have Apples Cert installed into openssl so you can use
```
openssl smime -in /path/to/your.mobileprovision -inform der -verify -noverify
```
