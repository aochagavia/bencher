import {
	BENCHER_CHAT_URL,
	BENCHER_GITHUB_URL,
	BENCHER_LINKEDIN_URL,
	BENCHER_WORDMARK,
} from "../../util/ext";

const Footer = () => (
	<footer class="footer" style="margin-top: 1rem;">
		<div class="container">
			<div class="columns is-vcentered is-mobile">
				<div class="column">
					<div class="content">
						<h4 class="title is-4">
							<a class="navbar-item" title="🐰 Bencher" href="/">
								<img src={BENCHER_WORDMARK} width="150" alt="🐰 Bencher" />
							</a>
						</h4>
						<h5 class="subtitle is-5">Continuous Benchmarking</h5>
						<nav class="level">
							<div class="level-left">
								<div class="level-item has-text-centered">
									<p>
										<a href="/docs/">Docs</a>
									</p>
								</div>
								<div class="level-item has-text-centered">
									<p>
										<a href="/pricing/">Pricing</a>
									</p>
								</div>
								<div class="level-item has-text-centered">
									<p>
										<a href="/help/">Help</a>
									</p>
								</div>
								<div class="level-item has-text-centered">
									<p>
										<a href="/learn/">Learn</a>
									</p>
								</div>
								<div class="level-item has-text-centered">
									<p>
										<a href="/legal/">Legal</a>
									</p>
								</div>
								<div class="level-item has-text-centered">
									<p>
										<a href="/brand/">Brand</a>
									</p>
								</div>
								<div class="level-item has-text-centered">
									<p>
										<a href="/sitemap-index.xml">Sitemap</a>
									</p>
								</div>
							</div>
						</nav>
						<nav class="level">
							<div class="level-left">
								<div class="level-item has-text-centered">
									<p>
										<a href="/legal/terms-of-use/">Terms of Use</a>
									</p>
								</div>
								<div class="level-item has-text-centered">
									<p>
										<a href="/legal/privacy/">Privacy Policy</a>
									</p>
								</div>
								<div class="level-item has-text-centered">
									<p>
										<a href="/legal/license/">License Agreement</a>
									</p>
								</div>
							</div>
						</nav>
					</div>
				</div>
				<div class="column is-narrow">
					<nav class="level is-mobile">
						<div class="level-item has-text-centered">
							<a
								class="navbar-item"
								href={BENCHER_GITHUB_URL}
								target="_blank"
								aria-label="GitHub"
								rel="noreferrer"
							>
								<span class="icon has-text-primary">
									<i class="fab fa-github fa-2x" aria-hidden="true" />
								</span>
							</a>
						</div>
						<div class="level-item has-text-centered">
							<a
								class="navbar-item"
								href={BENCHER_CHAT_URL}
								target="_blank"
								aria-label="Discord"
								rel="noreferrer"
							>
								<span class="icon has-text-primary">
									<i class="fab fa-discord fa-2x" aria-hidden="true" />
								</span>
							</a>
						</div>
						<div class="level-item has-text-centered">
							<a
								class="navbar-item"
								href={BENCHER_LINKEDIN_URL}
								target="_blank"
								aria-label="LinkedIn"
								rel="noreferrer"
							>
								<span class="icon has-text-primary">
									<i class="fab fa-linkedin fa-2x" aria-hidden="true" />
								</span>
							</a>
						</div>
					</nav>
				</div>
			</div>
			<div class="content">
				<p>© {new Date().getFullYear()} Bencher</p>
			</div>
		</div>
	</footer>
);

export default Footer;
