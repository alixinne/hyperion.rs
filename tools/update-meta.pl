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

if (($? >> 8) == 0) {
  while (my ($key, $value) = each %$details) {
    next unless exists $value->{template};

    my $basename = $key;
    $basename =~ s{_readme$}{};

    my $vars = {
      'cargo_run_output' => prefix_string('    ', `cargo run -q -p $basename -- --help`),
    };

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
    - Cargo.toml
