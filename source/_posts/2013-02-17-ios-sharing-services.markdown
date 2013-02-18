---
layout: post
title: "iOS Sharing Services"
date: 2012-02-24 08:10
comments: true
categories: [iOS]
---

<p>Ole Begemann has a nice post on <a href="http://oleb.net/blog/2012/02/what-ios-should-learn-from-android-and-windows-8/" title="What iOS Should Learn from Android and Windows 8">What iOS Should Learn from Android and Windows 8</a>. Specifically the post is about how Android, Windows 8 and iOS implement the sharing of things. The short version is that both Android and Windows 8 include a generic way to share things between applications and iOS has a very specific few ways to share only certain things.</p>
<p>There has been a lot of "Back to the Mac" talk in the last year however I think iOS could take a page from the Mac in this scenario. OS X has had the ability to share different types of media between applications using what are called <a href="https://developer.apple.com/library/mac/#documentation/Cocoa/Conceptual/SysServices/introduction.html" title="Services">Services</a>. You can probably find Services on your machine right now. Just select an image file in the Finder and then click Finder -&gt; Services from the Menu. This is what mine looks like<div class='p_embed p_image_embed'>
<a href="http://getfile0.posterous.com/getfile/files.posterous.com/temp-2012-02-23/DdJvkGffowduJEAoFlfeolvhoEyoxvmehtDuniCrnpkIvCvHcybEDeocnqhw/Services_Example.png.scaled1000.png"><img alt="Services_example" height="250" src="http://getfile1.posterous.com/getfile/files.posterous.com/temp-2012-02-23/DdJvkGffowduJEAoFlfeolvhoEyoxvmehtDuniCrnpkIvCvHcybEDeocnqhw/Services_Example.png.scaled500.png" width="500" /></a>
</div>
Take for example that "Upload with CloudApp". The Cloud.app on my machine has registered a Service with the OS that accepts an image file.There are even more Services installed for sharing a URL<div class='p_embed p_image_embed'>
<a href="http://getfile6.posterous.com/getfile/files.posterous.com/temp-2012-02-23/siHBsFpjtEAJtnamAAtkJkgCCgJcffusogHeHpiqdHgCImJxnhlCIHolmlbg/Services_URL_Example.png.scaled1000.png"><img alt="Services_url_example" height="315" src="http://getfile6.posterous.com/getfile/files.posterous.com/temp-2012-02-23/siHBsFpjtEAJtnamAAtkJkgCCgJcffusogHeHpiqdHgCImJxnhlCIHolmlbg/Services_URL_Example.png.scaled500.png" width="500" /></a>
</div>
Check out that Tweet Service. Thats provided by the official Twitter.app. It's like a Twitter Share Sheet right here in OS X Lion.</p>
<p><strong>The Technical Details</strong></p>
<p>Applications can register the Services they support by declaring them in the Info.plist. Lets go back to Twitter.app and see what that declaration looks like.</p>
<script src="https://gist.github.com/1898380.js?file=gistfile1.xml"></script>
<p>All this boils down to mean any selected text is sent to the Twitter.app tweetService method as a NSString from the Pasteboard. This is what the method signature of that Service probably looks like in thw Twitter.app AppDelegate</p>
<div class="CodeRay">
  <div class="code"><pre>- (void)tweetService:(NSPasteboard *)pboard userData:(NSString *)userData error:(NSString **)error;</pre></div>
</div>

<p><strong>What if iOS had this?</strong></p>
<p>Imagine if you could share a photo you took in <a href="http://campl.us/" title="Camera+">Camera+</a> with <a href="https://path.com/" title="Path">Path</a> without having to first save it to the Camera Roll and do a bunch of App Switching. The flow would go like this.</p>
<ol>
<li>Path registers a Service in it's Info.plist telling iOS that it supports image files.</li>
<li>Camera+ tells iOS it has an image to share.</li>
<li>iOS presents a list op App Services that support image files.</li>
<li>The user selectes a the Path App Service.</li>
<li>Path is launched with the image data passed to it's Service method that was registered in step #1.</li>
</ol>
<p>If that sounds a bit like D&eacute;j&agrave; vu, it's because it's very similar to how iOS already supports Documents.<div class='p_embed p_image_embed'>
<img alt="Photo_feb_23_10_33_52_pm" height="480" src="http://getfile0.posterous.com/getfile/files.posterous.com/temp-2012-02-23/GecqAJmyoFnJbspltfBrbzsrulonrjrqAzddwdjGGggEmmtJjdEkprHIJDIp/Photo_Feb_23_10_33_52_PM.png.scaled500.png" width="320" />
</div>
</p>
<p><strong>Does iOS already have this?</strong></p>
<p>One of the things Apple touted about iOS 5 was that it had some new "System Wide Services" including Dictionary lookup provided by iBooks. This iOS 5 preview from <a href="http://www.macworld.com/article/160322/2011/06/ios5.html" title="Macworld">Macworld</a> references it.<div class='p_embed p_image_embed'>
<a href="http://getfile0.posterous.com/getfile/files.posterous.com/temp-2012-02-23/JfEzyugarbDrHnmdfnGpJDImamqDIxohClfvjkHkCHDrHGBnnImhHwFfunjI/Dictionary_Service.png.scaled1000.png"><img alt="Dictionary_service" height="250" src="http://getfile9.posterous.com/getfile/files.posterous.com/temp-2012-02-23/JfEzyugarbDrHnmdfnGpJDImamqDIxohClfvjkHkCHDrHGBnnImhHwFfunjI/Dictionary_Service.png.scaled500.png" width="500" /></a>
</div>
</p>
<p>I don't see anything Service related in the iBooks Info.plist. That doesn't mean that the frameworks to support this doesn't already exist. Maybe it's just not exposed to us yet.</p>
