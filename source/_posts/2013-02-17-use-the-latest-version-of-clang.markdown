---
layout: post
title: "Use the latest version of Clang"
date: 2012-06-04 07:47
comments: true
categories: Xcode
---

<p><strong>Update</strong> <em>You must use an absolute path in step 3 or you could get a <a href="http://blog.hyperjeff.net/code?id=334">crash</a>. Thanks <a href="https://twitter.com/#!/hyperjeff">@hyperjeff</a></em></p>

<p>At the time of this writing the latest version of <a href="http://itunes.apple.com/us/app/xcode/id497799835?mt=12">Xcode</a> (4.3.2) was released on March 22nd 2012. The latest version of <a href="http://clang-analyzer.llvm.org/">Clang</a> (267) was released on June 1 2012. There have been 4 updates to Clang since Xcode 4.3.2 has shipped and those updates fix bugs and add better support for blocks. See the <a href="http://clang-analyzer.llvm.org/release_notes.html">Clang release notes</a>.</p>

<p>It is really easy to take advantage of these updates on your development machine or on your build servers.</p>

<h2>On your local development machine</h2>

<ol>
<li>Download Clang <code>curl -L http://bit.ly/LU4IZJ -o clang.tar.bz2</code></li>
<li>Untar it <code>tar -jxvf clang.tar.bz2</code></li>
<li>Tell Xcode about the updated Clang <code>sudo ./checker-267/set-xcode-analyzer --use-checker-build=$ABSOLUTE_PATH_TO_CLANG</code></li>
</ol>


<p>Now Xcode will use this updated version of Clang instead of the built in version. If you ever want to change back you run <code>sudo ./checker-267/set-xcode-analyzer --use-xcode-clang</code></p>

<h2>On your build servers</h2>

<p>The Jenkins <a href="https://wiki.jenkins-ci.org/display/JENKINS/Clang+Scan-Build+Plugin">Clang Scan-Build Plugin</a> has settings that define where Clang lives outside of Xcode. I install Clang in <code>/usr/local/bin/checker</code></p>

<p><img src="http://f.cl.ly/items/1W3P420H2Q3k2H413h1D/Screen%20Shot%202012-06-01%20at%205.18.53%20PM.png" alt="Clang Scan-Build Plugin Settings" /></p>

<p>With all that done you can now enjoy the benefits of the most up to date static code analysis.</p>

<p>Also, I love the graphs that the Clang Scan-Build Plugin creates <img src="http://f.cl.ly/items/28473r241g0b1b2O2s1b/Screen%20Shot%202012-06-01%20at%205.31.19%20PM.png" alt="" /></p>