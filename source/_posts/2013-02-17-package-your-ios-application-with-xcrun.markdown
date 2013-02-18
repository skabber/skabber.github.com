---
layout: post
title: "Package your iOS application with xcrun"
date: 2011-11-21 17:35
comments: true
categories: [iOS, Xcode]
---

<p>I have updated my example over the air distribution script to use "xcrun PackageApplication" instead of manually creating the Payload directory and zipping that.</p>
<p>This allows you to use the same flow to package and sign your application that Xcode does. It also allows you to choose the signing certificate and provisioning profile completely separate from what was defined in the Xcode project.</p>
<p>You can see how to use "xcrun PackageApplication" here</p>
<p><p><a href="https://gist.github.com/1385007">https://gist.github.com/1385007</a></p></p>
<p>Thanks to <a href="http://blog.octo.com/en/automating-over-the-air-deployment-for-iphone/">blog.octo.com</a> for finding this.&nbsp;</p>