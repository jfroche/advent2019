{ stableVersion }:
{
  nightly = {
    channel = "nightly";
    date = "2020-04-24";
  };
  stable = {
    channel = stableVersion;
  };
}
