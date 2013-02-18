---
layout: post
title: "Install a certificate via the command line."
date: 2012-01-10 22:02
comments: true
categories: [OS X, Keychain]
---
If you ever need to install a certificate into the OS X Keychain on a remote machine in such a way that Xcode can sign a build with it, then this command is for you.
```
security import /tmp/MyCertificates.p12 -k $HOME /Library/Keychains/login.keychain -P MyPassword -T /usr/bin/codesign
```
This allows you to not get that annoying "Always Allow" dialog the first time you try to use the cert to sign a build.