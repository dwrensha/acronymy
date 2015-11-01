@0xd78090a95ff14b08;

using Spk = import "/sandstorm/package.capnp";
# This imports:
#   $SANDSTORM_HOME/latest/usr/include/sandstorm/package.capnp
# Check out that file to see the full, documented package definition format.

const pkgdef :Spk.PackageDefinition = (
  # The package definition. Note that the spk tool looks specifically for the
  # "pkgdef" constant.

  id = "7m2fcfn7qdyexs3jmn6vrdngcryeuc8y4fa6jpyj4fgwh5tq27ph",

  manifest = (
    appTitle = (defaultText = "Acronymy"),
    appVersion = 6,  # Increment this for every release.
    appMarketingVersion = (defaultText = "2015.10.31"),

    metadata = (
      icons = (
        appGrid = (svg = embed "app-graphics/acronymy-128.svg"),
        grain = (svg = embed "app-graphics/acronymy-24.svg"),
        market = (svg = embed "app-graphics/acronymy-150.svg"),
      ),
      website = "https://dwrensha.ws/acronymy",
      codeUrl = "https://github.com/dwrensha/acronymy",
      license = (openSource = bsd2Clause),
      categories = [games,],
      author = (
        contactEmail = "david@sandstorm.io",
        pgpSignature = embed "pgp-signature",
      ),
      pgpKeyring = embed "pgp-keyring",
      description = (defaultText = embed "description.md"),
      shortDescription = (defaultText = "Word game"),
      screenshots = [(width = 909, height = 440, png = embed "screenshot.png")],
      changeLog = (defaultText = embed "changeLog.md"),
    ),

    actions = [
      # Define your "new document" handlers here.
      ( title = (defaultText = "New Acronymy Instance"),
        nounPhrase = (defaultText = "dictionary"),
        command = .initCommand
      )
    ],

    continueCommand = .continueCommand
  ),

  sourceMap = (
    # Here we defined where to look for files to copy into your package. The
    # `spk dev` command actually figures out what files your app needs
    # automatically by running it on a FUSE filesystem. So, the mappings
    # here are only to tell it where to find files that the app wants.
    searchPath = [
      ( sourcePath = "." ),  # Search this directory first.
      ( sourcePath = "/",    # Then search the system root directory.
        hidePaths = [ "home", "proc", "sys" ]
        # You probably don't want the app pulling files from these places,
        # so we hide them. Note that /dev, /var, and /tmp are implicitly
        # hidden because Sandstorm itself provides them.
      )
    ]
  ),

  fileList = "sandstorm-files.list",
  # `spk dev` will write a list of all the files your app uses to this file.
  # You should review it later, before shipping your app.

  alwaysInclude = []
  # Fill this list with more names of files or directories that should be
  # included in your package, even if not listed in sandstorm-files.list.
  # Use this to force-include stuff that you know you need but which may
  # not have been detected as a dependency during `spk dev`. If you list
  # a directory here, its entire contents will be included recursively.
);

const initCommand :Spk.Manifest.Command = (
  # Here we define the command used to start up your server.
  argv = ["/target/release/acronymy", "--init", "/data.db", "/var/data.db"],
  environ = [
    # Note that this defines the *entire* environment seen by your app.
    (key = "PATH", value = "/usr/local/bin:/usr/bin:/bin")
  ]
);


const continueCommand :Spk.Manifest.Command = (
  # Here we define the command used to start up your server.
  argv = ["/target/release/acronymy"],
  environ = [
    # Note that this defines the *entire* environment seen by your app.
    (key = "PATH", value = "/usr/local/bin:/usr/bin:/bin")
  ]
);

