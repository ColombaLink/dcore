
<div style="overflow: hidden;position: relative;">
<h1 id="title_text" class="articleTitle ">Using GPG to Encrypt Your Data</h1>
</div>

<p>Encryption helps protect your files during inter-host file transfers that use protocols that are not already encrypted—for example, when using&nbsp;<kbd>ftp</kbd>&nbsp;or when using <kbd>shiftc</kbd> without the <kbd>--secure</kbd> option. We recommend using the <span class="glossaryItem _tooltip_custom_glossary tooltipstered" onclick="$(this).tooltipster('content', glosarry_items[1]).tooltipster('show');" onmouseout="closeTooltip(this);">GNU</span> Privacy Guard (GPG), an Open Source OpenPGP-compatible encryption system.</p>

<p>GPG has been installed on Pleiades, Endeavour, and Lou in the /usr/bin/gpg directory. If you do not have GPG installed on the system(s) that you would like to use for transferring files, please see the <a href="http://www.gnupg.org" target="_blank&quot;">GPG website</a>.</p>

<h2>Choosing What Cipher to Use</h2>

<p>We recommend using the cipher AES256, which uses a 256-bit Advanced Encryption Standard (AES) key to encrypt the data. Information on AES can be found at the National Institute of Standards and Technology's <a href="http://csrc.nist.gov/" target="_blank">Computer Security Resource Center</a>.</p>

<p>You can set your cipher in one of the following ways:</p>

<ul>
	<li>Add <kbd>--cipher-algo AES256</kbd> to your ~/.gnupg/gpg.conf file.</li>
	<li>Add <kbd>--cipher-algo AES256</kbd> in the command line to override the default cipher, CAST5.</li>
</ul>

<h2>Examples</h2>

<p>If you choose not to add the <kbd>cipher-algo AES256</kbd> to your gpg.conf file, you can add <kbd>--cipher-algo AES256</kbd> on any of these simple example command lines to override the default cipher, CAST5.</p>

<h3>Creating an Encrypted File</h3>

<p>Both commands below are identical. They encrypt the test.out file and produce the encrypted version in the test.gpg file:</p>

<pre class=" language-bash"><code class=" language-bash"><span class="token operator">%</span> gpg <span class="token operator">--</span>output <span class="token function">test</span><span class="token punctuation">.</span>gpg <span class="token operator">--</span>symmetric <span class="token function">test</span><span class="token punctuation">.</span>out

<span class="token operator">%</span> gpg <span class="token operator">-</span>o <span class="token function">test</span><span class="token punctuation">.</span>gpg <span class="token operator">-</span>c <span class="token function">test</span><span class="token punctuation">.</span>out</code></pre>

<p>You will be prompted for a passphrase, which will be used later to decrypt the file.</p>

<h3>Decrypting a File</h3>

<p>The following command decrypts the test.gpg file and produces the test.out file:</p>

<pre class=" language-bash"><code class=" language-bash"><span class="token operator">%</span> gpg <span class="token operator">--</span>output <span class="token function">test</span><span class="token punctuation">.</span>out <span class="token operator">-</span>d <span class="token function">test</span><span class="token punctuation">.</span>gpg </code></pre>

<p>You will be prompted for the passphrase that you used to encrypt the file. If you don't use the <kbd>--output</kbd> option, the command output goes to STDOUT. If you don't use any flags, it will decrypt to a file without the .gpg suffix. For example, using the following command line would result in the decrypted data in a file named "test":</p>

<pre class=" language-bash"><code class=" language-bash"><span class="token operator">%</span> gpg <span class="token function">test</span><span class="token punctuation">.</span>gpg </code></pre>

<p></p>

<h2>Selecting a Passphrase</h2>

<p>Your passphrase should have sufficient information entropy. We suggest that you include five words of 5-10 letters in size, chosen at random, with spaces, special characters, and/or numbers embedded into the words.</p>

<p>You need to be able to recall the passphrase that was used to encrypt the file.</p>

<h2>Factors that Affect Encrypt/Decrypt Speed on NAS Filesystems</h2>

<p>We do not recommend using the <kbd>--armour</kbd> option for encrypting files that will be transferred to/from NAS systems. This option is mainly intended for sending binary data through email, not via transfer commands such as&nbsp;<kbd>ftp</kbd>&nbsp;or <kbd>shiftc</kbd>&nbsp;with the&nbsp;<kbd>-secure</kbd>&nbsp;option. The file size tends to be about 33% bigger than without this option, and encrypting the data takes about 10-15% longer.</p>

<p>The level of compression used when encrypting/decrypting affects the time required to complete the operation. There are three options for the compression algorithm: <kbd>none</kbd>, <kbd>zip</kbd>, and <kbd>zlib</kbd>.</p>

<ul>
	<li><kbd>--compress-algo none</kbd> or <kbd>--compress-algo 0</kbd></li>
	<li><kbd>--compress-algo zip</kbd> or <kbd>--compress-algo 1</kbd></li>
	<li><kbd>--compress-algo zlib</kbd> or <kbd>--compress-algo 2</kbd></li>
</ul>

<p>For example:</p>

<pre class=" language-bash"><code class=" language-bash"><span class="token operator">%</span> gpg <span class="token operator">--</span>output <span class="token function">test</span><span class="token punctuation">.</span>gpg <span class="token operator">--</span>compress<span class="token operator">-</span>algo zlib <span class="token operator">--</span>symmetric <span class="token function">test</span><span class="token punctuation">.</span>out </code></pre>

<p>If your data is not compressible, <kbd>--compress-algo 0</kbd> (<kbd>none</kbd>) gives you a performance increase of about 50% compared to <kbd>--compress-algo 1</kbd> or <kbd>--compress-algo 2</kbd>.</p>

<p>If your data is highly compressible, choosing the <kbd>zlib</kbd> or <kbd>zip</kbd> option will not only increase the speed by 20-50%, it will also reduce the file size by up to 20x. For example, in one test on a NAS system, a 517 megabyte (MB) highly compressible file was compressed to 30 MB.</p>

<p>The <kbd>zlib</kbd> option is not compatible with PGP 6.<em>x</em>, but neither is the cipher algorithm AES256. Using the <kbd>zlib</kbd> option is about 10% faster than using the <kbd>zip</kbd> option on a NAS system, and <kbd>zlib</kbd> compresses about 10% better than <kbd>zip</kbd>.</p>

<h2>Random Benchmark Data</h2>

<p>We tested the encryption/decryption speed of three different files (1 MB, 150 MB, and 517 MB) on NAS systems. The file used for the 1 MB test was an RPM file, presumably already compressed, since the resulting file sizes for the <kbd>none/zip/zlib</kbd> options were within 1% of each other. The 150 MB file was an ISO file, also assumed to be a compressed binary file for the same reasons. The 517 MB file was a text file. These runs were performed on a CXFS filesystem when many other users' jobs were running. The performance reported here is for reference only, and not the best or worst performance you can expect.</p>

<table class="zebra rollover" cellpadding="5" border="">
	<tbody>
		<tr>
			<th colspan="4">Using AES256 as the Cipher Algorithm</th>
		</tr>
		<tr>
			<td></td>
			<td><strong>1 MB File</strong></td>
			<td><strong>150 MB File</strong></td>
			<td><strong>517 MB File </strong></td>
		</tr>
		<tr>
			<td><strong>with --armour</strong></td>
			<td>~5.5 secs to encrypt</td>
			<td>~40 secs to encrypt</td>
			<td></td>
		</tr>
		<tr>
			<td><strong>without --armour</strong></td>
			<td>~4 secs to encrypt</td>
			<td>~35 secs to encrypt</td>
			<td></td>
		</tr>
		<tr>
			<td><strong>without --armour, zlib compression</strong></td>
			<td></td>
			<td>~33 secs to encrypt; ~28 secs to decrypt to file</td>
			<td>~33 secs, resultant file size ~30 MB; ~34 secs to decrypt to file</td>
		</tr>
		<tr>
			<td><strong>without --armour, zip compression</strong></td>
			<td></td>
			<td>~36 secs to encrypt; ~31 secs to decrypt to file</td>
			<td>~38 secs, resultant file size ~33 MB; ~34 secs to decrypt to file</td>
		</tr>
		<tr>
			<td><strong>without --armour, no compression</strong></td>
			<td></td>
			<td>~19 secs to encrypt; ~25 secs to decrypt to file</td>
			<td>~49 secs, resultant file size ~517 MB; ~75 secs to decrypt to file</td>
		</tr>
	</tbody>
</table>

