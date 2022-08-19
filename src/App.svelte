<script lang="ts">
  import { onMount } from 'svelte'
  import { invoke } from '@tauri-apps/api'
  import { Readability } from '@mozilla/readability'
  import { emit, listen } from '@tauri-apps/api/event'
  import type { LoginOutput } from '../src-tauri/bindings/LoginOutput'
  import type { IsLoginOutput } from '../src-tauri/bindings/IsLoginOutput'
  import type { ListOutput } from '../src-tauri/bindings/ListOutput'
  import type { SendInput } from '../src-tauri/bindings/SendInput'
  import type { Article } from '../src-tauri/bindings/Article'

  let authUrl: string | undefined
  let isLogin: boolean = false
  let articles: Article[] = []

  onMount(async () => {
    await listen('login', async () => {
      isLogin = true
      onLoggedIn()
    })

    // called from Rust
    const unlisten = await listen('readability-request', (event) => {
      // @ts-ignore
      const { content } = event.payload

      const parser = new DOMParser()
      const dom = parser.parseFromString(content.trim(), 'text/html')
      console.log('request', dom)

      const article = new Readability(dom, { debug: false }).parse()
      console.log('readable', article)

      // TODO: parse DOM to get image urls

      emit('readability-response', { article })
    })

    const isLoginOutput = await invoke<IsLoginOutput>('is_login')
    isLogin = isLoginOutput.isLogin
    console.log('isLogin', isLogin)
    if (isLogin) {
      onLoggedIn()
      return
    }

    const ret = await invoke<LoginOutput>('login')
    authUrl = ret.authUrl
  })

  const onLogoutClick = async () => {
    await invoke('logout')
  }

  const onLoggedIn = async () => {
    const ret = await invoke<ListOutput>('list')
    console.log(ret)
    articles = ret.articles
  }

  const onSendToKindleClick = async () => {
    const ret = await invoke<any>('send', { input: { articles } })
    console.log(ret)
  }
</script>

<h1>Send Pocket Article to Kindle</h1>

<!-- https://github.com/tauri-apps/tauri/issues/3830 -->
{#if authUrl && !isLogin}
  <a href={authUrl} target="_blank">
    <button>Log in</button>
  </a>
{/if}

{#if isLogin}
  <button on:click={onLogoutClick}>Logout</button>
{/if}

<button on:click={onSendToKindleClick}>Send to Kindle</button>

<div>
  {#each articles as article}
    <div>
      <h2>{article.title}</h2>
      {#if article.cover}
        <img src={article.cover} width="300" />
      {/if}
      <p>{article.contents}</p>
    </div>
  {/each}
</div>

<style>
</style>
