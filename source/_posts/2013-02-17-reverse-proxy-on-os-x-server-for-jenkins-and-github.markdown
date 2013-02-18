---
layout: post
title: "Reverse Proxy on OS X Server for Jenkins &amp; Github"
date: 2012-05-14 15:00
comments: true
categories: [Github, Jenkins, git,]
---

When I first started using Jenkins with Github I was using scm polling to check github for changes. This is bad, there is a much better way. Have github tell you when changes have been pushed via a Service Hook. The problem is that your Jenkins might be behind a private network. The solution is to use a Reverse Proxy.

Setting up a Reverse Proxy on OS X Lion is easy.

1. Download <a href="http://cl.ly/Gcc1">jenkinsProxy.plist</a>
2. Modify the url to your local Jenkins server
3. Move <code>jenkinsProxy.plist</code> into <code>/etc/apache2/webapps/</code>
4. Run <code>sudo webappctl start org.jenkins.proxy</code>
5. Point the Github Jenkins Service Hook [^fn]  to your external IP/Domain Name /github-webhook

And you're done. Every push to your github repo will ping your local Jenkins server to check for updates.
![](http://f.cl.ly/items/141O3B3j3V111u2n1Z0A/Screen%20Shot%202012-05-14%20at%204.01.41%20PM.png)

[^fn]: found in Admin -&gt; Service Hooks of your repo
