{ stableVersion }:
{
  nightly = {
    channel = "nightly";
    date = "2019-10-23";
  };
  stable = {
    channel = stableVersion;
  };
}
