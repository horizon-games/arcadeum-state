module.exports = {
  title: 'Arcadeum State',
  tagline: '',
  url: 'https://github.com/horizon-games/arcadeum-state',
  baseUrl: '/',
  favicon: 'img/favicon.ico',
  organizationName: 'arcadeum', // Usually your GitHub org/user name.
  projectName: 'arcadeum-state', // Usually your repo name.
  themeConfig: {
    navbar: {
      title: 'Arcadeum',
      logo: {
        alt: 'Arcadeum Logo',
        src: 'img/logo.svg',
      },
      links: [
        {to: 'docs/doc1', label: 'Docs', position: 'left'},
        {
          href: 'https://github.com/horizon-games/arcadeum-state',
          label: 'GitHub',
          position: 'right',
        },
      ],
    },
    footer: {
      style: 'dark',
      links: [
        {
          title: 'Docs',
          items: [
            {
              label: 'Style Guide',
              to: 'docs/doc1',
            },
            {
              label: 'Second Doc',
              to: 'docs/doc2',
            },
          ],
        },
        {
          title: 'Community',
          items: [
            {
              label: 'Discord',
              href: 'https://discord.gg/vPTDAzP',
            },
          ],
        },
        {
          title: 'Social',
          items: [
            {
              label: 'GitHub',
              href: 'https://github.com/horizon-games/arcadeum-state',
            },
          ],
        },
      ],
      copyright: `Copyright © ${new Date().getFullYear()} Horizon Blockchain Games, Inc. Built with Docusaurus.`,
    },
  },
  presets: [
    [
      '@docusaurus/preset-classic',
      {
        docs: {
          sidebarPath: require.resolve('./sidebars.js'),
          editUrl:
            'https://github.com/facebook/docusaurus/edit/master/website/',
        },
        theme: {
          customCss: require.resolve('./src/css/custom.css'),
        },
      },
    ],
  ],
};
