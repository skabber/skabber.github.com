---
layout: post
title: "iOS universal app to device specific app."
date: 2011-02-27 21:04
comments: true
categories: [iOS, Xcode]
---

<p>I recently had the need to deuniversalify an iOS app. This is a simple process but it took me longer than anticipated to find the right change to make. &nbsp;At first I thought it would be as simple as changing the&nbsp;<a href="http://developer.apple.com/library/ios/documentation/general/Reference/InfoPlistKeyReference/Articles/iPhoneOSKeys.html#//apple_ref/doc/uid/TP40009252-SW11" title="UIDeviceFamily">UIDeviceFamily</a>&nbsp;from "iPhone/iPad" back to "iPhone". &nbsp;That had no effect on the how the binary ran on the iPad. &nbsp;The iPad still ran the iPad specific code.</p>
<p><a href="http://developer.apple.com/library/ios/documentation/general/Reference/InfoPlistKeyReference/Articles/iPhoneOSKeys.html#//apple_ref/doc/uid/TP40009252-SW11" title="UIDeviceFamily"><div class='p_embed p_image_embed'>
<a href="http://getfile7.posterous.com/getfile/files.posterous.com/temp-2011-02-27/foeDpsfHhfvoBkpCpdhwAGfdzrebfhAimIbIHejarrvziinsaxphiaooHHAw/XcodeScreenSnapz004.png.scaled1000.png"><img alt="Xcodescreensnapz004" height="105" src="http://getfile6.posterous.com/getfile/files.posterous.com/temp-2011-02-27/foeDpsfHhfvoBkpCpdhwAGfdzrebfhAimIbIHejarrvziinsaxphiaooHHAw/XcodeScreenSnapz004.png.scaled500.png" width="500" /></a>
</div>
</a></p>
<p>I had to remove the <a href="http://developer.apple.com/library/ios/documentation/general/Reference/InfoPlistKeyReference/Articles/AboutInformationPropertyListFiles.html#//apple_ref/doc/uid/TP40009254-SW9" title="Creating device-specific keys">NSMainNibFile~iPad</a>&nbsp;and make sure that NSMainNibFile was pointing to the iPhone specific .xib.</p>
<p><a href="http://developer.apple.com/library/ios/documentation/general/Reference/InfoPlistKeyReference/Articles/iPhoneOSKeys.html#//apple_ref/doc/uid/TP40009252-SW11" title="UIDeviceFamily"><div class='p_embed p_image_embed'>
<img alt="Xcodescreensnapz005" height="49" src="http://getfile1.posterous.com/getfile/files.posterous.com/temp-2011-02-27/lClcnFkfbrmktrHdDDeEwAJqDlopmkzolGtntcvrrthuIkFJygGpcEiqijJx/XcodeScreenSnapz005.png.scaled500.png" width="488" />
</div>
</a></p>
<p>Then the app ran on the iPad using the iPhone screen size, exactly what I was after.</p>
<p>At the time of this writing, Google returns no results for "deuniversalify". That means I invented it :)</p>