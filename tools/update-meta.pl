#!/usr/bin/env perl
use v5.26.0;
use warnings;

use Cwd qw/getcwd/;
use File::Basename qw/dirname/;
use Template;
use YAML qw/Load/;

local $/;

my $details = do {
  my $yaml = <main::DATA>;
  Load($yaml)
};

die "must be run from the repository root\n" unless -d '.git';

my $failures = 0;

# Version updating
my $head_tag = `git tag --points-at HEAD`;

if (($? >> 8) == 0 && $head_tag) {
  $head_tag =~ s{\n*$}{};

  for my $toml_file (@{$details->{toml}->{files}}) {
    say STDERR "updating $toml_file";

    open my $fh, '+<:encoding(utf-8)', $toml_file;
    my $content = <$fh>;
    $content =~ s{^version.*$}{version = "$head_tag"}m;
    truncate $fh, 0;
    seek $fh, 0, 0;
    print $fh $content;
    close $fh;
  }
}

# README updating
my $tt = Template->new({ INCLUDE_PATH => '.' });

say STDERR "fetching help output";
my $vars = {
  'cargo_run_output' => prefix_string('    ', `cargo run -q -- --help`),
};

if (($? >> 8) == 0) {
  while (my ($key, $value) = each %$details) {
    next unless exists $value->{template};

    my $target = $value->{target};
    my $template = $value->{template};

    say STDERR "updating $target";

    # Read entire file
    open my $fh, '+<:encoding(utf-8)', $target
      or die "failed to open $target: $!";
    my $src = <$fh>;

    # Remove self-comment block
    $src =~ s{^//!.*\n}{}gm;

    # Template the output
    my $output = '';
    $tt->process(\$template, $vars, \$output)
      or die $tt->error(), "\n";

    # Write the output
    truncate $fh, 0;
    seek $fh, 0, 0;
    print $fh remove_trailing_whitespace(prefix_string('//! ', $output));
    print $fh $src;
    close $fh;
  }
} else {
  warn "not updating sources, cargo run failed\n";
}

my $old = getcwd();

while (my ($key, $value) = each %$details) {
  next unless exists $value->{readme};

  my $target = $value->{target};
  my $readme = $value->{readme};

  say STDERR "updating $readme";

  chdir(dirname($readme));
  my $output = `cargo readme`;
  chdir($old);

  if (($? >> 8) == 101) {
    warn "missing cargo-readme, aborting\n";
    $failures++;
    last;
  } elsif (($? >> 8) == 0) {
    open my $fh, '>', $readme;
    print $fh $output;
    close $fh;
  } else {
    warn "cargo-readme failed for $readme\n";
    $failures++;
  }
}

exit 1 if $failures;

sub prefix_string {
  my ($prefix, $string) = @_;
  $string =~ s{^}{$prefix}gm;
  $string
}

sub remove_trailing_whitespace {
  my ($s) = @_;
  $s =~ s{(?<!\s) $}{}gm;
  $s
}

__DATA__
toml:
  files:
    - hyperion/Cargo.toml
    - hyperiond/Cargo.toml
hyperion_readme:
  target: hyperion/src/lib.rs
  readme: hyperion/README.md
hyperiond_readme:
  target: hyperiond/src/main.rs
  readme: hyperiond/README.md
  template: |
    `hyperiond` is the Rust implementation of the
    [Hyperion](https://github.com/hyperion-project/hyperion) ambient lighting software. It is
    written from scratch both as an experiment and as a way to add more features.
    
    # Usage
    
    For now, the CLI is only able to start the hyperion server implementation:
    
        $ cargo run -- server --help
    [% cargo_run_output %]
    Logging is set using the HYPERION_LOG environment variable, which can be set to the desired
    logging level (trace, debug, info, warn, error). Note that this will affect logging of all
    crates, and if only hyperion logging is required, it should be filtered as such:
    `HYPERION_LOG=hyperion=level`. See the [env_logger crate docs](https://docs.rs/env_logger/0.6.1/env_logger/)
    for more details.
    
    # Development
    
    The source code in this folder is only responsible for the command-line interface and starting
    the server code, which is implemented in the [core crate](../hyperion)
    
    # Authors
    
    * [Vincent Tavernier](https://github.com/vtavernier)
    
    # License
    
    This source code is released under the [MIT-License](https://opensource.org/licenses/MIT)
