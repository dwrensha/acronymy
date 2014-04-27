@0x847d50a412ae7c63;

using Package = import "package.capnp";

const manifest :Package.Manifest = (
  appVersion = 2,

  actions = [(

    input = (none = void),
    title = (defaultText = "New Acronymy Instance"),

    command = (
      executablePath = "/acronymy",
      args = ["--init", "/data.db", "/var/data.db"]
    )
  )],

  continueCommand = (
    executablePath = "/acronymy"
  )
);